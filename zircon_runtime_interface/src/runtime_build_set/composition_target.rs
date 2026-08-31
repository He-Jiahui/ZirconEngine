use serde::{Deserialize, Serialize};

/// Product target that selected a frozen runtime module composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrRuntimeModuleCompositionTargetV1 {
    ClientRuntime,
    ServerRuntime,
    EditorHost,
}
