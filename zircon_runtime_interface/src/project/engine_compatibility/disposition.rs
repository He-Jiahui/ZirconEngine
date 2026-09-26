use serde::{Deserialize, Serialize};

/// 所选引擎对项目版本要求的预检判定，供 Editor 决定是否允许激活。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEngineCompatibilityDisposition {
    Compatible,
    ProjectRequiresNewerEngine,
    ProjectRequiresOlderEngine,
    /// 要求不匹配，且无法可靠归为需要更新或更旧的引擎。
    Incompatible,
}

impl ProjectEngineCompatibilityDisposition {
    pub const fn is_compatible(self) -> bool {
        matches!(self, Self::Compatible)
    }
}
