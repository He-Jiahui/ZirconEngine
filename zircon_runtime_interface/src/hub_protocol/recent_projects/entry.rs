use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::project::ProjectManifestSummary;

use super::HubRecentProjectsError;

/// One project entry in the Hub and Editor shared recent-project registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 最近项目列表中的展示记录，不承载打开授权或稳定项目身份。
pub struct HubRecentProjectV1 {
    pub summary: ProjectManifestSummary,
    pub path: PathBuf,
    pub last_opened_unix_ms: u64,
}

impl HubRecentProjectV1 {
    /// 以展示名称、词法路径和最近访问逻辑时间创建记录。
    pub fn new(
        summary: ProjectManifestSummary,
        path: impl Into<PathBuf>,
        last_opened_unix_ms: u64,
    ) -> Result<Self, HubRecentProjectsError> {
        let entry = Self {
            summary,
            path: path.into(),
            last_opened_unix_ms,
        };
        entry.validate()?;
        Ok(entry)
    }

    pub(crate) fn validate(&self) -> Result<(), HubRecentProjectsError> {
        if self.path.as_os_str().is_empty() {
            return Err(HubRecentProjectsError::EmptyProjectPath);
        }
        Ok(())
    }
}
