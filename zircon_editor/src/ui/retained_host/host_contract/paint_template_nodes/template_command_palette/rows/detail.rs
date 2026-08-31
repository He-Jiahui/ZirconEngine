use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, row_detail_rect};

mod style;

use style::command_row_detail_text_style;

pub(super) fn push_command_row_detail(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    if text.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = command_palette_metrics();
    let style = command_row_detail_text_style(color, &metrics);
    commands.push(HostPaintCommand::text(
        row_detail_rect(row_rect),
        Some(clip.clone()),
        order,
        text.to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
