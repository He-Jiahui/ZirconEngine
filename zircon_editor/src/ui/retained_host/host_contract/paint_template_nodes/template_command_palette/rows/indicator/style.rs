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
