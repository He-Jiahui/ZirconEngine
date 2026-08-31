use serde::{Deserialize, Serialize};

/// Optional module-selection profile embedded in a composition identity.
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
