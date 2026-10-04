use serde::{Deserialize, Serialize};

/// 文本请求和形状结果共用的方向语义；Auto 由排版服务按文本解析。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextDirection {
    #[default]
    Auto,
    LeftToRight,
    RightToLeft,
    Mixed,
}
