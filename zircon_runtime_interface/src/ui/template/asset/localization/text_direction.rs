use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 模板声明和本地化报告保留方向意图；Auto 尚未在此边界决定最终文本排版方向。
pub enum UiTextDirection {
    #[default]
    Auto,
    LeftToRight,
    RightToLeft,
}
