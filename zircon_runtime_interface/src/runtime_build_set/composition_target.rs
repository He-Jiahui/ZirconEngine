use serde::{Deserialize, Serialize};

/// Product target that selected a frozen runtime module composition.
/// 选择冻结 Runtime 模块组合的产品目标。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrRuntimeModuleCompositionTargetV1 {
    ClientRuntime,
    ServerRuntime,
    EditorHost,
}
