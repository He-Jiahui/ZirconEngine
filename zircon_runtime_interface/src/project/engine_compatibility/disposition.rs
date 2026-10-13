use serde::{Deserialize, Serialize};

/// Whether the selected engine satisfies a project's declared semantic-version range.
/// 所选引擎对项目版本要求的预检判定，供 Editor 决定是否允许激活。
///
/// # 使用时机
/// 由 [`crate::project::assess_project_engine_compatibility`] 在清单预检阶段产生；Editor 在项目激活前检查
/// [`is_compatible`](Self::is_compatible)，并可用方向变体组织错误反馈。
///
/// # 约束
/// 两个方向变体只在要求区间边界能证明方向时返回；无法化为非空稳定版本区间的情况归为 `Incompatible`。
///
/// # 参考
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 与
/// `zircon_runtime_interface/src/project/tests/directional_range.rs` 覆盖具体分支。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEngineCompatibilityDisposition {
    /// 当前引擎版本满足整个项目要求。
    Compatible,
    /// 当前版本低于要求区间的下界，项目需要更新的引擎。
    ProjectRequiresNewerEngine,
    /// 当前版本高于要求区间的上界，项目需要较旧的引擎。
    ProjectRequiresOlderEngine,
    /// 要求不匹配，且无法可靠归为需要更新或更旧的引擎。
    Incompatible,
}

impl ProjectEngineCompatibilityDisposition {
    /// 只有 `Compatible` 表示满足项目的 semver 要求。
    pub const fn is_compatible(self) -> bool {
        matches!(self, Self::Compatible)
    }
}
