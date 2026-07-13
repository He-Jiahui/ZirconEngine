use serde::{Deserialize, Serialize};

/// Selects the runtime family whose modules and platform capabilities are assembled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTargetMode {
    ClientRuntime,
    ServerRuntime,
    EditorHost,
}
