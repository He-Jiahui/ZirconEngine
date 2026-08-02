use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectManifestSummary;

#[cfg(test)]
use zircon_runtime_interface::project::PROJECT_MANIFEST_FORMAT_VERSION;

pub const RECENT_PROJECT_LIMIT: usize = 8;

/// Hub project history entry whose display identity is always the shared manifest summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub summary: ProjectManifestSummary,
    pub path: PathBuf,
    pub last_opened_unix_ms: u64,
}

impl RecentProject {
    pub fn from_summary(
        summary: ProjectManifestSummary,
        path: impl Into<PathBuf>,
        last_opened_unix_ms: u64,
    ) -> Self {
        Self {
            summary,
            path: path.into(),
            last_opened_unix_ms,
        }
    }

    pub fn from_project_path(
        path: impl Into<PathBuf>,
        last_opened_unix_ms: u64,
    ) -> Result<Self, HubError> {
        let path = path.into();
        let bytes = fs::read(path.join("zircon-project.toml"))?;
        let summary = ProjectManifestSummary::parse_toml_bytes(&bytes)?.value;
        Ok(Self::from_summary(summary, path, last_opened_unix_ms))
    }

    pub fn with_now(path: impl Into<PathBuf>) -> Result<Self, HubError> {
        Self::from_project_path(path, now_unix_ms())
    }

    pub fn refresh_summary(&mut self) -> Result<(), HubError> {
        self.summary =
            Self::from_project_path(self.path.clone(), self.last_opened_unix_ms)?.summary;
        Ok(())
    }

    pub fn display_name(&self) -> &str {
        &self.summary.name
    }

    #[cfg(test)]
    pub fn fixture(
        display_name: impl Into<String>,
        path: impl Into<PathBuf>,
        last_opened_unix_ms: u64,
    ) -> Self {
        Self {
            summary: ProjectManifestSummary {
                name: display_name.into(),
                engine_version_req: None,
                default_scene: "res://scenes/main.scene.toml".to_string(),
                format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            },
            path: path.into(),
            last_opened_unix_ms,
        }
    }
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}
