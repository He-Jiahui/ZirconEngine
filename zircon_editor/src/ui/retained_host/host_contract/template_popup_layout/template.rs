use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::dropdown::dropdown_option_popup_frame_with_height_within;
use super::metrics::{dropdown_option_row_height, popup_row_height, popup_rows_height};
use super::rows::popup_row_frame;

pub(crate) fn template_option_popup_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if template_option_rows_use_projected_frame(node) {
        return (row_count > 0).then_some(control_frame.clone());
    }
    let row_height = dropdown_option_row_height(control_frame);
    let popup_height = popup_rows_height(node, row_count, row_height)?;
    dropdown_option_popup_frame_with_height_within(control_frame, popup_height, bounds)
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
        popup_row_height(node, &popup, row_count)?
    } else {
        dropdown_option_row_height(control_frame)
    };
    popup_row_frame(node, &popup, row_count, row, row_height)
}

pub(crate) fn template_option_rows_use_projected_frame(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "DropdownPopup")
        || matches!(node.component_role.as_str(), "dropdown-popup")
}
