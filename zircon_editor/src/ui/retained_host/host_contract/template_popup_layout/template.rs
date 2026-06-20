use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::dropdown::dropdown_option_popup_frame_within;
use super::metrics::{dropdown_option_row_height, menu_item_row_height};

pub(crate) fn template_option_popup_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if template_option_rows_use_projected_frame(node) {
        return (row_count > 0).then_some(control_frame.clone());
    }
    dropdown_option_popup_frame_within(control_frame, row_count, bounds)
}

pub(crate) fn template_option_row_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if row >= row_count {
        return None;
    }
    let popup = template_option_popup_frame_within(node, control_frame, row_count, bounds)?;
    let row_height = if template_option_rows_use_projected_frame(node) {
        menu_item_row_height(&popup, row_count)?
    } else {
        dropdown_option_row_height(control_frame)
    };
    Some(FrameRect {
        x: popup.x,
        y: popup.y + row as f32 * row_height,
        width: popup.width,
        height: row_height,
    })
}

pub(crate) fn template_option_rows_use_projected_frame(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "DropdownPopup")
        || matches!(node.component_role.as_str(), "dropdown-popup")
}
