use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::layout::command_palette_metrics;

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
    if let Some(background) = style.background {
        let metrics = command_palette_metrics();
        commands.push(HostPaintCommand::quad(
            row_rect.clone(),
            Some(clip.clone()),
            order,
            Some(background),
            style.outline,
            if style.outline.is_some() { 1.0 } else { 0.0 },
            metrics.row_radius,
            opacity,
        ));
    }
}
