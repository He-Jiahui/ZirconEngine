use serde::{Deserialize, Serialize};

/// 选择水平或纵排 shaping 请求；它不包含最终 UI 行布局策略。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextWritingMode {
    #[default]
    HorizontalTopToBottom,
    VerticalRightToLeft,
}
