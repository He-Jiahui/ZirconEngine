use serde::{Deserialize, Serialize};

/// Dynamic-session policy selected independently from the module graph profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZrRuntimeSessionProfileV1 {
    Runtime,
    RuntimePipelined,
    Editor,
    Dev,
    Minimal,
    Headless,
}
