use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::layout::command_palette_metrics;

mod style;

use style::command_row_surface_style;

pub(super) fn push_command_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchPopupRowStyle,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = command_palette_metrics();
    let surface_style = command_row_surface_style(style, &metrics);
    let Some(style) = surface_style else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        row_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.fill),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}
