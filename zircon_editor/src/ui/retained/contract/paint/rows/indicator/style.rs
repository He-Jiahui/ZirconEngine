//! 匹配标记样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::super::super::data::TemplatePaneOptionData;
use super::color::command_row_match_indicator_color;

pub(super) struct CommandRowMatchIndicatorStyle {
    pub fill: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_row_match_indicator_style(
    option: &TemplatePaneOptionData,
    radius: f32,
) -> CommandRowMatchIndicatorStyle {
    CommandRowMatchIndicatorStyle {
        fill: command_row_match_indicator_color(option),
        border: None,
        border_width: 0.0,
        radius,
    }
}
