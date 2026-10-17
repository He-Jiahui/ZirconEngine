use serde::{Deserialize, Serialize};

/// Optional module-selection profile embedded in a composition identity.
/// 组合身份中可选的模块选择 profile；它只描述模块图，不替代动态会话策略。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrRuntimeModuleProfileV1 {
    Minimal,
    Client2d,
    Client3d,
    Editor,
    Dev,
    Server,
}
