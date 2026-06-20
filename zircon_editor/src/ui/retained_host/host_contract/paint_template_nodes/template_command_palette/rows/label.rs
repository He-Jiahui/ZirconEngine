use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{row_label_rect, FONT_SIZE, LINE_HEIGHT};
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
    commands.push(HostPaintCommand::text(
        row_label_rect(row_rect),
        Some(clip.clone()),
        order,
        text,
        color,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
