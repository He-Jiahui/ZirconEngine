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

    /// 按逻辑时钟推进打开时间，再以规范化路径合并项目并清除同路径墓碑；revision 随此次变更递增。
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

    /// 从当前投影移除路径并记录墓碑，使较旧的合并输入不能把用户已删除的项目重新带回。
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

    /// 文件解码和写回前都用此检查拒绝超限、重复或未按规范排序的投影与墓碑。
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
#[path = "tests/registry_performance_tests.rs"]
mod performance_tests;
