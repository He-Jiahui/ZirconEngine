use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::render_commands::HostPaintCommand;
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
    push_command_row_label(
        commands,
        row_rect,
        clip,
        order + 2,
        option.label.to_string(),
        style.text,
        opacity,
    );
}
