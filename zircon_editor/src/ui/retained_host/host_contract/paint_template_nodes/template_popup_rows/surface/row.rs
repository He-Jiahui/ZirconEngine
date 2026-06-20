use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::metrics::{POPUP_ROW_ORDER_OFFSET, POPUP_ROW_SELECTED_MARK_WIDTH};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_surface(
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
            order + POPUP_ROW_ORDER_OFFSET + 1,
            Some(background),
            None,
            0.0,
            3.0,
            opacity,
        ));
    }
    if let Some(selection_mark) = style.selection_mark {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: row_rect.x,
                y: row_rect.y + 4.0,
                width: POPUP_ROW_SELECTED_MARK_WIDTH,
                height: (row_rect.height - 8.0).max(1.0),
            },
            Some(clip.clone()),
            order + POPUP_ROW_ORDER_OFFSET + 2,
            Some(selection_mark),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
