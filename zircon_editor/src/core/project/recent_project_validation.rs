//! 给欢迎页区分缺失来源、无效清单和必须迁移等状态，支持打开动作的展示约束；实际启动仍需重新检查。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecentProjectValidation {
    Valid,
    RequiresMigration,
    Missing,
    InvalidManifest,
    #[default]
    InvalidProject,
}
