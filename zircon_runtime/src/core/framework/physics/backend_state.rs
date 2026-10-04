use serde::{Deserialize, Serialize};

/// 表示后端已禁用、不可用或就绪，供诊断界面解释选择结果；模拟模式另行决定是否步进。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsBackendState {
    Disabled,
    Unavailable,
    #[default]
    Ready,
}
