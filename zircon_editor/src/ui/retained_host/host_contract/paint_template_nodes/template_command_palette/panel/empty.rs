use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, empty_text_rect};
use super::super::palette::command_palette_palette;
use super::super::text::command_palette_empty_message;

mod style;

use style::command_palette_empty_text_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_empty_message(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = command_palette_metrics();
    let palette = command_palette_palette();
    let style = command_palette_empty_text_style(&palette, &metrics);
    commands.push(HostPaintCommand::wrapped_text(
        empty_text_rect(rect),
        Some(clip.clone()),
        order,
        command_palette_empty_message().to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
