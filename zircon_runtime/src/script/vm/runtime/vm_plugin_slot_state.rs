use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 协调器在加载、重载、卸载和失败路径上公布的槽位阶段。
pub enum VmPluginSlotState {
    #[default]
    Active,
    Reloading,
    Unloading,
    Failed,
}

impl VmPluginSlotState {
    /// 返回日志、错误和结构化报告使用的稳定标签。
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Reloading => "reloading",
            Self::Unloading => "unloading",
            Self::Failed => "failed",
        }
    }
}
