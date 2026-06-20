use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::layout::{selection_mark_rect, ROW_RADIUS, SELECTION_MARK_RADIUS};

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
        commands.push(HostPaintCommand::quad(
            row_rect.clone(),
            Some(clip.clone()),
            order,
            Some(background),
            None,
            0.0,
            ROW_RADIUS,
            opacity,
        ));
    }
    if let Some(selection_mark) = style.selection_mark {
        commands.push(HostPaintCommand::quad(
            selection_mark_rect(row_rect),
            Some(clip.clone()),
            order + 1,
            Some(selection_mark),
            None,
            0.0,
            SELECTION_MARK_RADIUS,
            opacity,
        ));
    }
}
