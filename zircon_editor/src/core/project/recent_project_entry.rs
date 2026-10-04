//! 把共享最近项目登记和只读校验状态投影给启动会话；摘要用于展示，校验状态不替代后续正式预检及项目准入。
use serde::{Deserialize, Serialize};
use zircon_runtime_interface::hub_protocol::HubRecentProjectV1;
use zircon_runtime_interface::project::ProjectManifestSummary;

use super::RecentProjectValidation;

/// Recent-project identity projected from the authoritative project manifest summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProjectEntry {
    pub summary: ProjectManifestSummary,
    pub path: String,
    pub last_opened_unix_ms: u64,
    #[serde(default)]
    pub validation: RecentProjectValidation,
}

impl RecentProjectEntry {
    pub(crate) fn from_shared(
        project: HubRecentProjectV1,
        validation: RecentProjectValidation,
    ) -> Self {
        Self {
            summary: project.summary,
            // TODO: [CR-EDITOR-PROJECT-0003] 确认共享登记是否在写入前拒绝非统一码路径；此有损投影也进入校验和后续项目动作，需沿存储准入补身份保真证据。
            path: project.path.to_string_lossy().into_owned(),
            last_opened_unix_ms: project.last_opened_unix_ms,
            validation,
        }
    }
}
