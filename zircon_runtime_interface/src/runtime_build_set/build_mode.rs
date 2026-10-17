use serde::{Deserialize, Serialize};

/// Cargo build mode shared by artifacts in an internal Runtime BuildSet.
/// 内部 Runtime BuildSet 中所有产物共用的 Cargo 构建模式。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZrRuntimeBuildModeV1 {
    Debug,
    Release,
    Profiling,
}
