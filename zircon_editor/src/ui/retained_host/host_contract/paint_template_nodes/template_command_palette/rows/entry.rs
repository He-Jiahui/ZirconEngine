use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::{row_label_order, row_match_indicator_order};
use super::detail::push_command_row_detail;
use super::indicator::push_command_row_match_indicator;
use super::label::push_command_row_label;
use super::style::command_row_style;
use super::surface::push_command_row_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = command_row_style(option);
    push_command_row_surface(commands, row_rect, clip, order, style, opacity);
    push_command_row_match_indicator(
        commands,
        option,
        row_rect,
        clip,
        row_match_indicator_order(order),
        opacity,
    );
    push_command_row_label(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.label.to_string(),
        style.text,
        opacity,
    );
    push_command_row_detail(
        commands,
        row_rect,
        clip,
        row_label_order(order),
        option.description.to_string(),
        style.shortcut,
        opacity,
    );
}
