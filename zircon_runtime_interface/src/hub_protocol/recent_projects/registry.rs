use std::collections::{btree_map::Entry, BTreeMap};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{hub_recent_project_path_key, HubRecentProjectV1, HubRecentProjectsError};
use crate::hub_protocol::HubProtocolVersionV1;

pub const HUB_RECENT_PROJECT_LIMIT_V1: usize = 8;
pub const HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1: usize = 64;

/// Durable evidence that an entry was deliberately removed from the rebuildable recent projection.
///
/// Tombstones are keyed by the same normalized display-path key as v1 entries. Stable project and
/// filesystem identity remain preflight-owned; this projection never promotes a display path into
/// a project identity authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubRecentProjectTombstoneV1 {
    path_key: String,
    deleted_logical_unix_ms: u64,
}

impl HubRecentProjectTombstoneV1 {
    fn new(path_key: String, deleted_logical_unix_ms: u64) -> Self {
        Self {
            path_key,
            deleted_logical_unix_ms,
        }
    }

    pub fn path_key(&self) -> &str {
        &self.path_key
    }

    pub const fn deleted_logical_unix_ms(&self) -> u64 {
        self.deleted_logical_unix_ms
    }
}

/// Canonical on-disk document shared between the Hub and Editor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubRecentProjectsV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub revision: u64,
    pub projects: Vec<HubRecentProjectV1>,
    pub tombstones: Vec<HubRecentProjectTombstoneV1>,
}

impl HubRecentProjectsV1 {
    pub fn new(projects: impl IntoIterator<Item = HubRecentProjectV1>) -> Self {
        let projects = merge_hub_recent_projects(projects, []);
        Self {
            protocol_version: HubProtocolVersionV1,
            revision: u64::from(!projects.is_empty()),
            projects,
            tombstones: Vec::new(),
        }
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn record(
        &mut self,
        mut project: HubRecentProjectV1,
    ) -> Result<(), HubRecentProjectsError> {
        project.last_opened_unix_ms = self.next_logical_timestamp(project.last_opened_unix_ms)?;
        let path_key = hub_recent_project_path_key(&project.path);
        let projects = merge_hub_recent_projects(self.projects.iter().cloned(), [project]);
        self.bump_revision()?;
        self.projects = projects;
        self.tombstones
            .retain(|tombstone| tombstone.path_key != path_key);
        Ok(())
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), HubRecentProjectsError> {
        let path_key = hub_recent_project_path_key(path);
        let project_exists = self
            .projects
            .iter()
            .any(|project| hub_recent_project_path_key(&project.path) == path_key);
        if !project_exists
            && self
                .tombstones
                .iter()
                .any(|tombstone| tombstone.path_key == path_key)
        {
            return Ok(());
        }
        let deleted_logical_unix_ms = self.next_tombstone_timestamp()?;
        self.projects
            .retain(|project| hub_recent_project_path_key(&project.path) != path_key);
        self.tombstones
            .retain(|tombstone| tombstone.path_key != path_key);
        self.tombstones.push(HubRecentProjectTombstoneV1::new(
            path_key,
            deleted_logical_unix_ms,
        ));
        self.canonicalize_tombstones();
        self.bump_revision()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), HubRecentProjectsError> {
        if self.projects.len() > HUB_RECENT_PROJECT_LIMIT_V1 {
            return Err(HubRecentProjectsError::TooManyEntries {
                limit: HUB_RECENT_PROJECT_LIMIT_V1,
            });
        }
        if self.tombstones.len() > HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1 {
            return Err(HubRecentProjectsError::TooManyTombstones {
                limit: HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1,
            });
        }

        let mut keys = BTreeMap::new();
        let mut previous_project = None;
        let mut canonical_order = true;
        for project in &self.projects {
            project.validate()?;
            let key = hub_recent_project_path_key(&project.path);
            match keys.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(());
                }
                Entry::Occupied(entry) => {
                    return Err(HubRecentProjectsError::DuplicateProjectPath {
                        path_key: entry.key().clone(),
                    });
                }
            }
            if canonical_order
                && previous_project
                    .is_some_and(|previous| !is_canonical_successor(previous, project))
            {
                canonical_order = false;
            }
            previous_project = Some(project);
        }

        if !canonical_order {
            return Err(HubRecentProjectsError::NonCanonicalOrder);
        }
        let mut tombstone_keys = BTreeMap::new();
        let mut previous_tombstone = None;
        for tombstone in &self.tombstones {
            if tombstone.path_key.is_empty() {
                return Err(HubRecentProjectsError::EmptyTombstonePathKey);
            }
            if keys.contains_key(&tombstone.path_key) {
                return Err(HubRecentProjectsError::TombstoneOverlapsProject {
                    path_key: tombstone.path_key.clone(),
                });
            }
            match tombstone_keys.entry(tombstone.path_key.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(());
                }
                Entry::Occupied(entry) => {
                    return Err(HubRecentProjectsError::DuplicateTombstonePathKey {
                        path_key: entry.key().clone(),
                    });
                }
            }
            if previous_tombstone.is_some_and(|previous: &HubRecentProjectTombstoneV1| {
                !is_canonical_tombstone_successor(previous, tombstone)
            }) {
                return Err(HubRecentProjectsError::NonCanonicalTombstoneOrder);
            }
            previous_tombstone = Some(tombstone);
        }
        Ok(())
    }

    fn bump_revision(&mut self) -> Result<(), HubRecentProjectsError> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(HubRecentProjectsError::RevisionExhausted)?;
        Ok(())
    }

    fn next_logical_timestamp(&self, proposed_unix_ms: u64) -> Result<u64, HubRecentProjectsError> {
        match self.latest_logical_timestamp() {
            Some(current) => current
                .checked_add(1)
                .map(|next| proposed_unix_ms.max(next))
                .ok_or(HubRecentProjectsError::LogicalClockExhausted),
            None => Ok(proposed_unix_ms),
        }
    }

    fn next_tombstone_timestamp(&self) -> Result<u64, HubRecentProjectsError> {
        self.latest_logical_timestamp().map_or(Ok(1), |current| {
            current
                .checked_add(1)
                .ok_or(HubRecentProjectsError::LogicalClockExhausted)
        })
    }

    fn latest_logical_timestamp(&self) -> Option<u64> {
        self.projects
            .iter()
            .map(|project| project.last_opened_unix_ms)
            .chain(
                self.tombstones
                    .iter()
                    .map(|tombstone| tombstone.deleted_logical_unix_ms),
            )
            .max()
    }

    fn canonicalize_tombstones(&mut self) {
        self.tombstones.sort_by(|left, right| {
            right
                .deleted_logical_unix_ms
                .cmp(&left.deleted_logical_unix_ms)
                .then_with(|| left.path_key.cmp(&right.path_key))
        });
        self.tombstones
            .truncate(HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1);
    }
}

fn is_canonical_successor(previous: &HubRecentProjectV1, current: &HubRecentProjectV1) -> bool {
    previous.last_opened_unix_ms > current.last_opened_unix_ms
        || (previous.last_opened_unix_ms == current.last_opened_unix_ms
            && previous.path <= current.path)
}

fn is_canonical_tombstone_successor(
    previous: &HubRecentProjectTombstoneV1,
    current: &HubRecentProjectTombstoneV1,
) -> bool {
    previous.deleted_logical_unix_ms > current.deleted_logical_unix_ms
        || (previous.deleted_logical_unix_ms == current.deleted_logical_unix_ms
            && previous.path_key <= current.path_key)
}

impl Default for HubRecentProjectsV1 {
    fn default() -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            revision: 0,
            projects: Vec::new(),
            tombstones: Vec::new(),
        }
    }
}

/// Merges two registry snapshots without retaining a host-specific authority field.
///
/// The newest timestamp wins; identical timestamps use manifest name as a deterministic tie
/// break. The resulting collection is path-deduplicated, ordered, and bounded.
pub fn merge_hub_recent_projects<I, J>(left: I, right: J) -> Vec<HubRecentProjectV1>
where
    I: IntoIterator<Item = HubRecentProjectV1>,
    J: IntoIterator<Item = HubRecentProjectV1>,
{
    let mut projects = BTreeMap::<String, HubRecentProjectV1>::new();
    for project in left.into_iter().chain(right) {
        let key = hub_recent_project_path_key(&project.path);
        match projects.get(&key) {
            Some(existing) if existing.last_opened_unix_ms > project.last_opened_unix_ms => {}
            Some(existing)
                if existing.last_opened_unix_ms == project.last_opened_unix_ms
                    && existing.summary.name <= project.summary.name => {}
            _ => {
                projects.insert(key, project);
            }
        }
    }
    let mut merged = projects.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .last_opened_unix_ms
            .cmp(&left.last_opened_unix_ms)
            .then_with(|| left.path.cmp(&right.path))
    });
    merged.truncate(HUB_RECENT_PROJECT_LIMIT_V1);
    merged
}

#[cfg(test)]
mod performance_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::project::{ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION};

    use super::{
        hub_recent_project_path_key, merge_hub_recent_projects, HubProtocolVersionV1,
        HubRecentProjectV1, HubRecentProjectsError, HubRecentProjectsV1,
        HUB_RECENT_PROJECT_LIMIT_V1,
    };

    const PERF_NAME_BYTES: usize = 16 * 1024;
    const PERF_SAMPLE_PAIRS: usize = 21;
    const PERF_VALIDATIONS_PER_SAMPLE: usize = 128;

    #[test]
    fn single_pass_validation_preserves_duplicate_error_precedence() {
        let registry = HubRecentProjectsV1 {
            protocol_version: HubProtocolVersionV1,
            revision: 1,
            projects: vec![
                recent_project("Older", "E:/Projects/Shared", 1),
                recent_project("Newer", "E:/Projects/Other", 2),
                recent_project("Duplicate", "e:\\Projects\\Shared\\", 0),
            ],
            tombstones: Vec::new(),
        };

        assert_eq!(
            registry.validate(),
            Err(HubRecentProjectsError::DuplicateProjectPath {
                path_key: "e:/projects/shared".to_string(),
            })
        );
    }

    #[test]
    fn removal_persists_a_tombstone_and_a_later_open_advances_its_logical_clock() {
        let mut registry = HubRecentProjectsV1::default();
        registry
            .record(recent_project("Game", "E:/Projects/Game", 100))
            .unwrap();
        registry.remove("e:/projects/game/").unwrap();

        assert_eq!(registry.revision(), 2);
        assert!(registry.projects.is_empty());
        assert_eq!(registry.tombstones[0].path_key(), "e:/projects/game");
        assert_eq!(registry.tombstones[0].deleted_logical_unix_ms(), 101);

        registry
            .record(recent_project("Game", "E:/Projects/Game", 1))
            .unwrap();

        assert_eq!(registry.revision(), 3);
        assert!(registry.tombstones.is_empty());
        assert_eq!(registry.projects[0].last_opened_unix_ms, 102);
        assert!(registry.validate().is_ok());
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn canonical_validation_borrows_registry_entries() {
        let registry = canonical_fixture();
        assert!(legacy_is_valid(&registry));
        assert!(registry.validate().is_ok());
        let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);

        for sample in 0..PERF_SAMPLE_PAIRS {
            let (legacy, optimized) = if sample % 2 == 0 {
                (
                    measure_batch(|| legacy_is_valid(&registry)),
                    measure_batch(|| registry.validate().is_ok()),
                )
            } else {
                let optimized = measure_batch(|| registry.validate().is_ok());
                let legacy = measure_batch(|| legacy_is_valid(&registry));
                (legacy, optimized)
            };
            legacy_ns.push(legacy);
            optimized_ns.push(optimized);
        }

        let legacy_p50 = percentile(&legacy_ns, 50);
        let legacy_p95 = percentile(&legacy_ns, 95);
        let optimized_p50 = percentile(&optimized_ns, 50);
        let optimized_p95 = percentile(&optimized_ns, 95);
        println!(
            "PERF_RESULT runtime_interface06_recent_registry_validation legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} entries={HUB_RECENT_PROJECT_LIMIT_V1} name_bytes_per_entry={PERF_NAME_BYTES} samples={PERF_SAMPLE_PAIRS} validations_per_sample={PERF_VALIDATIONS_PER_SAMPLE} legacy_entry_clones=8 optimized_entry_clones=0 legacy_path_normalizations=16 optimized_path_normalizations=8 legacy_accepted_path_key_clones=8 optimized_accepted_path_key_clones=0 legacy_entry_visits=16 optimized_entry_visits=8"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(25),
            "optimized P95 {optimized_p95}ns must be at most 25% of legacy P95 {legacy_p95}ns"
        );
    }

    fn canonical_fixture() -> HubRecentProjectsV1 {
        HubRecentProjectsV1::new((0..HUB_RECENT_PROJECT_LIMIT_V1).map(|index| {
            HubRecentProjectV1::new(
                ProjectManifestSummary {
                    name: format!("project-{index}-{}", "x".repeat(PERF_NAME_BYTES)),
                    engine_version_req: None,
                    default_scene: "res://scenes/main.scene.toml".to_string(),
                    format_version: PROJECT_MANIFEST_FORMAT_VERSION,
                    project_guid: None,
                },
                format!("E:/Projects/Project-{index}"),
                (HUB_RECENT_PROJECT_LIMIT_V1 - index) as u64,
            )
            .expect("performance fixture project must be valid")
        }))
    }

    fn recent_project(name: &str, path: &str, last_opened_unix_ms: u64) -> HubRecentProjectV1 {
        HubRecentProjectV1::new(
            ProjectManifestSummary {
                name: name.to_string(),
                engine_version_req: None,
                default_scene: "res://scenes/main.scene.toml".to_string(),
                format_version: PROJECT_MANIFEST_FORMAT_VERSION,
                project_guid: None,
            },
            path,
            last_opened_unix_ms,
        )
        .expect("recent project fixture must be valid")
    }

    fn legacy_is_valid(registry: &HubRecentProjectsV1) -> bool {
        let mut keys = BTreeMap::new();
        for project in &registry.projects {
            if project.validate().is_err() {
                return false;
            }
            let key = hub_recent_project_path_key(&project.path);
            if keys.insert(key.clone(), ()).is_some() {
                return false;
            }
        }
        merge_hub_recent_projects(registry.projects.iter().cloned(), []) == registry.projects
    }

    fn measure_batch(mut measure: impl FnMut() -> bool) -> u64 {
        let started = Instant::now();
        for _ in 0..PERF_VALIDATIONS_PER_SAMPLE {
            black_box(measure());
        }
        (started.elapsed().as_nanos() / PERF_VALIDATIONS_PER_SAMPLE as u128) as u64
    }

    fn percentile(samples: &[u64], percentile: usize) -> u64 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered
            .len()
            .saturating_mul(percentile)
            .div_ceil(100)
            .saturating_sub(1);
        ordered[rank]
    }
}
