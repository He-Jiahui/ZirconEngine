use serde::{Deserialize, Serialize};

/// Dynamic-session policy selected independently from the module graph profile.
/// 独立于模块图 profile 选择的动态会话策略；回执中二者分别占用不同字段。
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
