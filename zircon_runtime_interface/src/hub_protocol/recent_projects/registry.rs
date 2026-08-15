use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{hub_recent_project_path_key, HubRecentProjectV1, HubRecentProjectsError};
use crate::hub_protocol::HubProtocolVersionV1;

pub const HUB_RECENT_PROJECT_LIMIT_V1: usize = 8;

/// Canonical on-disk document shared between the Hub and Editor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HubRecentProjectsV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub projects: Vec<HubRecentProjectV1>,
}

impl HubRecentProjectsV1 {
    pub fn new(projects: impl IntoIterator<Item = HubRecentProjectV1>) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            projects: merge_hub_recent_projects(projects, []),
        }
    }

    pub fn record(&mut self, project: HubRecentProjectV1) {
        self.projects = merge_hub_recent_projects(self.projects.drain(..), [project]);
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) {
        let path_key = hub_recent_project_path_key(path);
        self.projects
            .retain(|project| hub_recent_project_path_key(&project.path) != path_key);
    }

    pub fn validate(&self) -> Result<(), HubRecentProjectsError> {
        if self.projects.len() > HUB_RECENT_PROJECT_LIMIT_V1 {
            return Err(HubRecentProjectsError::TooManyEntries {
                limit: HUB_RECENT_PROJECT_LIMIT_V1,
            });
        }

        let mut keys = BTreeMap::new();
        for project in &self.projects {
            project.validate()?;
            let key = hub_recent_project_path_key(&project.path);
            if keys.insert(key.clone(), ()).is_some() {
                return Err(HubRecentProjectsError::DuplicateProjectPath { path_key: key });
            }
        }

        if merge_hub_recent_projects(self.projects.iter().cloned(), []) != self.projects {
            return Err(HubRecentProjectsError::NonCanonicalOrder);
        }
        Ok(())
    }
}

impl Default for HubRecentProjectsV1 {
    fn default() -> Self {
        Self::new([])
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
