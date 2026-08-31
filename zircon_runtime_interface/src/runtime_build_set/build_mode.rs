use serde::{Deserialize, Serialize};

/// Cargo build mode shared by artifacts in an internal Runtime BuildSet.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZrRuntimeBuildModeV1 {
    Debug,
    Release,
    Profiling,
}
