use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime_interface::project::{
    ProjectManifestSummary, RenderedProjectTemplate, render_project_template,
};

use super::filesystem::{
    canonical_project_root, validate_canonical_existing_project_root, validate_creation_target,
};
use super::{
    CreatedProject, NewProjectDraft, OpenedProject, ProjectAuthorityError, ProjectProbe,
    RecentProjectEntry, RecentProjectValidation, StoredRecentProjectEntry, StoredStartupSession,
};

const RECENT_PROJECT_LIMIT: usize = 8;
static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(1);

/// Headless-safe owner for Editor project identity, creation, opening, and recents.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectAuthority;

impl ProjectAuthority {
    pub(crate) fn decode_startup_session(
        &self,
        mut value: serde_json::Value,
    ) -> Result<StoredStartupSession, ProjectAuthorityError> {
        self.migrate_legacy_recent_project_summaries(&mut value)?;
        serde_json::from_value(value)
            .map_err(|source| ProjectAuthorityError::SessionDecode { source })
    }

    pub(crate) fn encode_startup_session(
        &self,
        session: &StoredStartupSession,
    ) -> Result<serde_json::Value, ProjectAuthorityError> {
        serde_json::to_value(session)
            .map_err(|source| ProjectAuthorityError::SessionEncode { source })
    }

    pub fn create_project(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        let target = draft.validate_for_creation()?;
        let rendered = render_project_template(draft.template.pack_id(), &draft.project_name)?;
        self.create_rendered_project(&target, rendered)
    }

    pub(crate) fn create_rendered_project(
        &self,
        target: &Path,
        rendered: RenderedProjectTemplate,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        validate_creation_target(target)?;
        let parent = target
            .parent()
            .ok_or_else(|| ProjectAuthorityError::ProjectMissing {
                path: target.to_path_buf(),
            })?;
        fs::create_dir_all(parent)
            .map_err(|source| ProjectAuthorityError::io("create project parent", parent, source))?;
        let transaction = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
        let stem = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project");
        let staging = parent.join(format!(
            ".{stem}.zircon-staging-{}-{transaction}",
            std::process::id()
        ));
        let backup = parent.join(format!(
            ".{stem}.zircon-backup-{}-{transaction}",
            std::process::id()
        ));

        let mut staging_created = false;
        let result = (|| {
            fs::create_dir(&staging).map_err(|source| {
                ProjectAuthorityError::io("create project staging directory", &staging, source)
            })?;
            staging_created = true;
            for entry in rendered.entries {
                let destination = entry.path.join_to(&staging);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|source| {
                        ProjectAuthorityError::io("create template directory", parent, source)
                    })?;
                }
                fs::write(&destination, entry.bytes).map_err(|source| {
                    ProjectAuthorityError::io("write template entry", &destination, source)
                })?;
            }
            let staging_paths = ProjectPaths::from_root(&staging).map_err(|source| {
                ProjectAuthorityError::io("resolve staging project paths", &staging, source)
            })?;
            staging_paths.ensure_derived_layout().map_err(|source| {
                ProjectAuthorityError::io("create staging derived layout", &staging, source)
            })?;
            let manifest_path = staging.join("zircon-project.toml");
            let manifest = ProjectManifest::load(&manifest_path)?;
            manifest.save(&manifest_path)?;

            let replaced_empty_target = target.exists();
            commit_staged_directory(
                &staging,
                target,
                &backup,
                replaced_empty_target,
                |from, to| fs::rename(from, to),
            )?;
            let root = match canonical_project_root(target) {
                Ok(root) => root,
                Err(error) => {
                    rollback_committed_project(
                        &staging,
                        target,
                        &backup,
                        replaced_empty_target,
                        |from, to| fs::rename(from, to),
                    )?;
                    return Err(error);
                }
            };
            let project = match ProjectManager::open(&root) {
                Ok(project) => project,
                Err(source) => {
                    rollback_committed_project(
                        &staging,
                        target,
                        &backup,
                        replaced_empty_target,
                        |from, to| fs::rename(from, to),
                    )?;
                    return Err(source.into());
                }
            };
            if let Err(error) = finalize_empty_target_backup(target, &backup, replaced_empty_target)
            {
                drop(project);
                rollback_committed_project(
                    &staging,
                    target,
                    &backup,
                    replaced_empty_target,
                    |from, to| fs::rename(from, to),
                )?;
                return Err(error);
            }
            let summary = project.manifest().summary();
            Ok(CreatedProject {
                root,
                summary,
                project,
            })
        })();

        let preserve_rollback_artifacts = matches!(
            &result,
            Err(ProjectAuthorityError::PostCommitRollbackFailed { .. })
        );
        if result.is_err() {
            cleanup_failed_transaction_staging(
                &staging,
                preserve_rollback_artifacts,
                staging_created,
            );
        }
        result
    }

    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        let root = self.resolve_existing_project_root(path)?;
        Ok(OpenedProject::new(ProjectManager::open(root)?))
    }

    pub fn probe_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ProjectProbe, ProjectAuthorityError> {
        let root = self.resolve_existing_project_root(path)?;
        let paths = ProjectPaths::from_root(&root)
            .map_err(|source| ProjectAuthorityError::io("resolve project paths", &root, source))?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        Ok(ProjectProbe::new(root, manifest.summary()))
    }

    pub fn probe_draft(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<ProjectProbe, ProjectAuthorityError> {
        self.probe_project(draft.project_root()?)
    }

    pub fn resolve_existing_project_root(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<PathBuf, ProjectAuthorityError> {
        let root = canonical_project_root(path.as_ref())?;
        validate_canonical_existing_project_root(&root)?;
        Ok(root)
    }

    pub fn validate_recent_project(&self, path: &str) -> RecentProjectValidation {
        let Ok(root) = canonical_project_root(Path::new(path)) else {
            return RecentProjectValidation::InvalidProject;
        };
        if !root.exists() {
            return RecentProjectValidation::Missing;
        }
        if validate_canonical_existing_project_root(&root).is_err() {
            return RecentProjectValidation::Missing;
        }
        let Ok(paths) = ProjectPaths::from_root(&root) else {
            return RecentProjectValidation::InvalidProject;
        };
        match ProjectManifest::load(paths.manifest_path()) {
            Ok(_) => RecentProjectValidation::Valid,
            Err(_) => RecentProjectValidation::InvalidManifest,
        }
    }

    pub fn remember_recent_project(
        &self,
        session: &mut StoredStartupSession,
        path: &str,
        summary: ProjectManifestSummary,
        now_unix_ms: u64,
    ) {
        session.last_project_path = Some(path.to_string());
        session.recent_projects.retain(|entry| entry.path != path);
        session.recent_projects.push(StoredRecentProjectEntry {
            summary,
            path: path.to_string(),
            last_opened_unix_ms: now_unix_ms,
        });
        session.recent_projects.sort_by(|left, right| {
            right
                .last_opened_unix_ms
                .cmp(&left.last_opened_unix_ms)
                .then_with(|| left.path.cmp(&right.path))
        });
        session.recent_projects.truncate(RECENT_PROJECT_LIMIT);
    }

    pub fn forget_recent_project(&self, session: &mut StoredStartupSession, path: &str) {
        session.recent_projects.retain(|entry| entry.path != path);
        if session.last_project_path.as_deref() == Some(path) {
            session.last_project_path = session
                .recent_projects
                .first()
                .map(|entry| entry.path.clone());
        }
    }

    pub fn recent_projects_with_validation<F>(
        &self,
        session: &StoredStartupSession,
        mut validate: F,
    ) -> Vec<RecentProjectEntry>
    where
        F: FnMut(&str) -> RecentProjectValidation,
    {
        session
            .recent_projects
            .iter()
            .map(|entry| RecentProjectEntry {
                summary: entry.summary.clone(),
                path: entry.path.clone(),
                last_opened_unix_ms: entry.last_opened_unix_ms,
                validation: validate(&entry.path),
            })
            .collect()
    }

    fn migrate_legacy_recent_project_summaries(
        &self,
        session: &mut serde_json::Value,
    ) -> Result<(), ProjectAuthorityError> {
        let Some(recent_projects) = session
            .get_mut("recent_projects")
            .and_then(serde_json::Value::as_array_mut)
        else {
            return Ok(());
        };

        let mut migrated = Vec::with_capacity(recent_projects.len());
        for mut entry in std::mem::take(recent_projects) {
            let Some(record) = entry.as_object_mut() else {
                migrated.push(entry);
                continue;
            };
            if record.contains_key("summary") {
                migrated.push(entry);
                continue;
            }
            let Some(path) = record
                .get("path")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            let Ok(project) = self.probe_project(&path) else {
                continue;
            };
            let summary = serde_json::to_value(project.summary())
                .map_err(|source| ProjectAuthorityError::SessionEncode { source })?;
            record.insert("summary".to_string(), summary);
            migrated.push(entry);
        }
        *recent_projects = migrated;
        Ok(())
    }
}

pub(super) fn commit_staged_directory<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), ProjectAuthorityError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    if replace_empty_target {
        rename(target, backup).map_err(|source| {
            ProjectAuthorityError::io("stage empty target rollback", target, source)
        })?;
        match directory_is_empty(backup) {
            Ok(true) => {}
            Ok(false) => {
                let commit_source = std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "project target became non-empty during creation",
                );
                rename(backup, target).map_err(|restore_source| {
                    ProjectAuthorityError::CommitRollbackFailed {
                        target: target.to_path_buf(),
                        backup: backup.to_path_buf(),
                        commit_source,
                        restore_source,
                    }
                })?;
                return Err(ProjectAuthorityError::TargetNotEmpty {
                    path: target.to_path_buf(),
                });
            }
            Err(commit_source) => {
                if let Err(restore_source) = rename(backup, target) {
                    return Err(ProjectAuthorityError::CommitRollbackFailed {
                        target: target.to_path_buf(),
                        backup: backup.to_path_buf(),
                        commit_source,
                        restore_source,
                    });
                }
                return Err(ProjectAuthorityError::io(
                    "recheck empty project target before commit",
                    target,
                    commit_source,
                ));
            }
        }
    }

    if let Err(commit_source) = rename(staging, target) {
        if replace_empty_target {
            if let Err(restore_source) = rename(backup, target) {
                return Err(ProjectAuthorityError::CommitRollbackFailed {
                    target: target.to_path_buf(),
                    backup: backup.to_path_buf(),
                    commit_source,
                    restore_source,
                });
            }
        }
        return Err(ProjectAuthorityError::io(
            "commit project template",
            target,
            commit_source,
        ));
    }

    Ok(())
}

fn directory_is_empty(path: &Path) -> std::io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().transpose()?.is_none())
}

pub(super) fn rollback_committed_project<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), ProjectAuthorityError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    // Return the newly published directory to the transaction path so the caller's ordinary
    // error cleanup removes only this creation attempt.
    rename(target, staging).map_err(|source| ProjectAuthorityError::PostCommitRollbackFailed {
        from: target.to_path_buf(),
        to: staging.to_path_buf(),
        backup: replace_empty_target.then(|| backup.to_path_buf()),
        source,
    })?;
    if replace_empty_target {
        rename(backup, target).map_err(|source| {
            ProjectAuthorityError::PostCommitRollbackFailed {
                from: backup.to_path_buf(),
                to: target.to_path_buf(),
                backup: Some(backup.to_path_buf()),
                source,
            }
        })?;
    }
    Ok(())
}

pub(super) fn finalize_empty_target_backup(
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
) -> Result<(), ProjectAuthorityError> {
    if !replace_empty_target {
        return Ok(());
    }

    match fs::remove_dir(backup) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => match directory_is_empty(backup) {
            Ok(false) => Err(ProjectAuthorityError::TargetNotEmpty {
                path: target.to_path_buf(),
            }),
            Err(inspect_source) if inspect_source.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(true) | Err(_) => Err(ProjectAuthorityError::io(
                "finalize empty project target backup",
                backup,
                source,
            )),
        },
    }
}

pub(super) fn cleanup_failed_transaction_staging(
    staging: &Path,
    preserve_rollback_artifacts: bool,
    staging_created: bool,
) {
    // A failed staging creation has no ownership of a pre-existing path, so cleanup may only
    // remove a directory created by this transaction and not retained for rollback recovery.
    if staging_created && !preserve_rollback_artifacts {
        remove_transaction_path(staging);
    }
}

fn remove_transaction_path(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn canonical_project_resolution_does_not_repeat_link_component_validation() {
        let source = include_str!("authority.rs");
        let repeated_validation = ["validate_existing_project_root", "(&root)"].concat();

        assert!(!source.contains(&repeated_validation));
    }
}
