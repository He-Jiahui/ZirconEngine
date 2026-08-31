use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use zircon_runtime_interface::hub_protocol::{
    hub_recent_project_path_key, HubRecentProjectV1, HubRecentProjectsError,
    HubRecentProjectsStore, HubRecentProjectsStoreError, HubRecentProjectsV1,
    HubRecentProjectsWritePolicy,
};

use super::metadata::normalize_project_root;
use super::RecentProject;

// Hub reconciliation runs on its background action path and may wait briefly for a concurrent
// post-Ready Editor projection, but it never inherits an unbounded OS mutex wait.
const HUB_RECENT_PROJECTS_RECONCILIATION_WAIT: Duration = Duration::from_millis(250);
const HUB_RECENT_PROJECTS_CAS_RETRY_LIMIT: u8 = 4;

/// The Hub's last synchronized view of the shared recent-project projection.
///
/// The revision is deliberately carried beside the display rows. Consumers use it only to decide
/// whether a deletion derived from old local state may still be applied; it is not a project ID.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedRecentProjectsSnapshot {
    revision: u64,
    projects: Vec<RecentProject>,
}

impl SharedRecentProjectsSnapshot {
    fn from_registry(registry: HubRecentProjectsV1) -> Self {
        Self {
            revision: registry.revision(),
            projects: registry
                .projects
                .into_iter()
                .map(recent_project_from_shared)
                .collect(),
        }
    }

    fn from_projects(projects: Vec<RecentProject>) -> Self {
        Self {
            revision: 0,
            projects,
        }
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn projects(&self) -> &[RecentProject] {
        &self.projects
    }

    pub fn into_projects(self) -> Vec<RecentProject> {
        self.projects
    }
}

/// Loads the strict, versioned Hub/Editor shared recent-project registry as a rebuildable
/// projection. Corruption and oversized files return an empty projection instead of blocking Hub.
pub fn load_shared_recent_projects(
    registry_path: impl AsRef<Path>,
) -> Result<Vec<RecentProject>, SharedRecentProjectsError> {
    Ok(load_shared_recent_projects_snapshot(registry_path)?.into_projects())
}

/// Reads the shared projection together with the revision needed for safe future reconciliation.
pub fn load_shared_recent_projects_snapshot(
    registry_path: impl AsRef<Path>,
) -> Result<SharedRecentProjectsSnapshot, SharedRecentProjectsError> {
    let registry = HubRecentProjectsStore::new(registry_path.as_ref())
        .load_projection()?
        .registry()
        .clone();
    Ok(SharedRecentProjectsSnapshot::from_registry(registry))
}

/// Normalizes an in-memory Hub history through the same v1 DTO merge rules as the shared file.
pub fn merge_recent_project_entries<I, J>(
    left: I,
    right: J,
) -> Result<Vec<RecentProject>, SharedRecentProjectsError>
where
    I: IntoIterator<Item = RecentProject>,
    J: IntoIterator<Item = RecentProject>,
{
    let contract_path = Path::new("HubConfig.recent_projects");
    let projects = left
        .into_iter()
        .chain(right)
        .map(|project| shared_project_from_recent(contract_path, project))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HubRecentProjectsV1::new(projects)
        .projects
        .into_iter()
        .map(recent_project_from_shared)
        .collect())
}

/// Reconciles Hub changes against the current shared Hub/Editor registry in one bounded
/// transaction. A stalled writer yields a typed projection failure instead of blocking the Hub.
///
/// `previous_hub_projects` is the last snapshot synchronized into this Hub process. Comparing it
/// with `hub_projects` lets external Editor changes win unless the Hub changed that project after
/// the snapshot, so stale Hub memory cannot revive an Editor-side removal.
pub fn reconcile_shared_recent_projects(
    registry_path: impl AsRef<Path>,
    previous_hub_projects: &[RecentProject],
    hub_projects: &[RecentProject],
) -> Result<Vec<RecentProject>, SharedRecentProjectsError> {
    let previous_hub_projects =
        SharedRecentProjectsSnapshot::from_projects(previous_hub_projects.to_vec());
    Ok(reconcile_shared_recent_projects_snapshot(
        registry_path,
        &previous_hub_projects,
        hub_projects,
    )?
    .into_projects())
}

/// Reconciles Hub changes through a revisioned compare-and-update transaction.
///
/// A failed CAS re-reads the canonical projection and recalculates the same user-derived delta.
/// On a stale deletion, a project changed by another process is retained; an explicit Hub record
/// remains an intentional new open and may supersede a tombstone.
pub fn reconcile_shared_recent_projects_snapshot(
    registry_path: impl AsRef<Path>,
    previous_hub_snapshot: &SharedRecentProjectsSnapshot,
    hub_projects: &[RecentProject],
) -> Result<SharedRecentProjectsSnapshot, SharedRecentProjectsError> {
    let registry_path = registry_path.as_ref();
    let hub_projects = hub_projects
        .iter()
        .cloned()
        .map(|project| shared_project_from_recent(registry_path, project))
        .collect::<Result<Vec<_>, _>>()?;
    let previous_by_key = previous_hub_snapshot
        .projects()
        .iter()
        .cloned()
        .map(|project| (hub_recent_project_path_key(&project.path), project))
        .collect::<std::collections::BTreeMap<_, _>>();
    let current_by_key = hub_projects
        .iter()
        .cloned()
        .map(|project| (hub_recent_project_path_key(&project.path), project))
        .collect::<std::collections::BTreeMap<_, _>>();
    let removed_keys = previous_by_key
        .keys()
        .filter(|path_key| !current_by_key.contains_key(*path_key))
        .cloned()
        .collect::<Vec<_>>();
    let recorded_projects = current_by_key
        .iter()
        .filter_map(|(path_key, project)| {
            previous_by_key
                .get(path_key)
                .is_none_or(|previous| recent_project_changed(previous, project))
                .then(|| project.clone())
        })
        .collect::<Vec<_>>();
    if removed_keys.is_empty() && recorded_projects.is_empty() {
        return load_shared_recent_projects_snapshot(registry_path);
    }

    let store = HubRecentProjectsStore::new(registry_path);
    for _ in 0..HUB_RECENT_PROJECTS_CAS_RETRY_LIMIT {
        let projection = store.load_projection()?;
        let expected_revision = projection.registry().revision();
        let projection_matches_snapshot = expected_revision == previous_hub_snapshot.revision();
        match store.compare_and_update(
            HubRecentProjectsWritePolicy::with_timeout(HUB_RECENT_PROJECTS_RECONCILIATION_WAIT),
            expected_revision,
            |registry| {
                for path_key in &removed_keys {
                    let current = registry
                        .projects
                        .iter()
                        .find(|project| hub_recent_project_path_key(&project.path) == *path_key);
                    let unchanged_since_snapshot =
                        previous_by_key.get(path_key).is_some_and(|previous| {
                            current
                                .is_some_and(|current| !recent_project_changed(previous, current))
                        });
                    if projection_matches_snapshot || unchanged_since_snapshot {
                        let previous = previous_by_key
                            .get(path_key)
                            .expect("removed key always originates from the prior snapshot");
                        registry.remove(&previous.path)?;
                    }
                }
                for project in &recorded_projects {
                    registry.record(project.clone())?;
                }
                Ok(())
            },
        ) {
            Ok(mutation) => {
                return Ok(SharedRecentProjectsSnapshot::from_registry(
                    mutation.registry().clone(),
                ));
            }
            Err(HubRecentProjectsStoreError::RevisionConflict { .. }) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(SharedRecentProjectsError::CasRetryExhausted {
        attempts: HUB_RECENT_PROJECTS_CAS_RETRY_LIMIT,
    })
}

fn shared_project_from_recent(
    registry_path: &Path,
    project: RecentProject,
) -> Result<HubRecentProjectV1, SharedRecentProjectsError> {
    HubRecentProjectV1::new(
        project.summary,
        normalize_project_root(project.path),
        project.last_opened_unix_ms,
    )
    .map_err(|source| SharedRecentProjectsError::Contract {
        path: registry_path.to_path_buf(),
        source,
    })
}

fn recent_project_from_shared(project: HubRecentProjectV1) -> RecentProject {
    RecentProject::from_summary(project.summary, project.path, project.last_opened_unix_ms)
}

fn recent_project_changed(previous: &RecentProject, current: &HubRecentProjectV1) -> bool {
    previous.summary != current.summary
        || previous.last_opened_unix_ms != current.last_opened_unix_ms
}

#[derive(Debug, Error)]
pub enum SharedRecentProjectsError {
    #[error("shared recent-project registry `{path}` violates the v1 contract: {source}")]
    Contract {
        path: PathBuf,
        #[source]
        source: HubRecentProjectsError,
    },
    #[error(transparent)]
    Store(#[from] HubRecentProjectsStoreError),
    #[error(
        "shared recent-project registry changed during all {attempts} bounded reconciliation attempts"
    )]
    CasRetryExhausted { attempts: u8 },
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use zircon_runtime_interface::hub_protocol::{
        HubRecentProjectV1, HubRecentProjectsStore, HubRecentProjectsWritePolicy,
    };

    use super::{
        load_shared_recent_projects, reconcile_shared_recent_projects,
        reconcile_shared_recent_projects_snapshot, SharedRecentProjectsSnapshot,
    };
    use crate::projects::RecentProject;

    #[test]
    fn reconciliation_merges_hub_history_with_the_shared_v1_registry() {
        let root = temporary_root("merge");
        let registry_path = root.join("recent_projects.json");

        let first = reconcile_shared_recent_projects(
            &registry_path,
            &[],
            &[RecentProject::fixture("Old", "E:/Projects/Game", 1)],
        )
        .expect("write initial shared history");
        let second = reconcile_shared_recent_projects(
            &registry_path,
            &first,
            &[
                RecentProject::fixture("Current", "e:/projects/game/", 9),
                RecentProject::fixture("Other", "E:/Projects/Other", 2),
            ],
        )
        .expect("merge shared history");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 2);
        assert_eq!(second[0].summary.name, "Current");
        assert_eq!(
            load_shared_recent_projects(&registry_path).expect("read shared history")[0]
                .summary
                .name,
            "Current"
        );
        std::fs::remove_dir_all(root).expect("remove shared history fixture");
    }

    #[test]
    fn stale_hub_snapshot_does_not_restore_an_editor_removed_project() {
        let root = temporary_root("delete");
        let registry_path = root.join("recent_projects.json");
        let initial = reconcile_shared_recent_projects(
            &registry_path,
            &[],
            &[RecentProject::fixture("Game", "E:/Projects/Game", 1)],
        )
        .expect("write initial shared history");
        let removed = reconcile_shared_recent_projects(&registry_path, &initial, &[])
            .expect("record Editor-side removal");
        let after_stale_hub_persist =
            reconcile_shared_recent_projects(&registry_path, &initial, &initial)
                .expect("reconcile stale Hub snapshot");

        assert!(removed.is_empty());
        assert!(after_stale_hub_persist.is_empty());
        std::fs::remove_dir_all(root).expect("remove shared history fixture");
    }

    #[test]
    fn stale_hub_delete_preserves_an_externally_updated_project() {
        let root = temporary_root("stale-delete");
        let registry_path = root.join("recent_projects.json");
        let initial = reconcile_shared_recent_projects_snapshot(
            &registry_path,
            &SharedRecentProjectsSnapshot::default(),
            &[RecentProject::fixture("Game", "E:/Projects/Game", 1)],
        )
        .expect("write initial shared history");
        HubRecentProjectsStore::new(&registry_path)
            .update(
                HubRecentProjectsWritePolicy::with_timeout(Duration::from_millis(50)),
                |registry| {
                    registry.record(HubRecentProjectV1::new(
                        RecentProject::fixture("External", "E:/Projects/Game", 9).summary,
                        "E:/Projects/Game",
                        9,
                    )?)
                },
            )
            .expect("external editor update");

        let reconciled = reconcile_shared_recent_projects_snapshot(&registry_path, &initial, &[])
            .expect("stale Hub delete must rebase safely");

        assert_eq!(reconciled.projects()[0].summary.name, "External");
        std::fs::remove_dir_all(root).expect("remove shared history fixture");
    }

    fn temporary_root(label: &str) -> std::path::PathBuf {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR").expect(
            "shared recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR",
        );
        std::path::PathBuf::from(target_directory).join(format!(
            "zircon-hub-shared-recents-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ))
    }
}
