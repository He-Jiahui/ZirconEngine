use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, row_label_rect};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_command_row_label(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    opacity: f32,
) {
    if text.is_empty() || intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = command_palette_metrics();
    commands.push(HostPaintCommand::text(
        row_label_rect(row_rect),
        Some(clip.clone()),
        order,
        text,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
