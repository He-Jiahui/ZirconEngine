use serde::{Deserialize, Serialize};

/// 字形呈现策略请求；Auto 由 TextLayoutService 决定后再交给渲染路径。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextRenderMode {
    #[default]
    Auto,
    Native,
    Sdf,
    Msdf,
    Mtsdf,
}
