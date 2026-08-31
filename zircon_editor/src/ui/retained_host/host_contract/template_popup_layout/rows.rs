use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{popup_row_height, popup_row_layout};

pub(crate) fn menu_item_row_frame(
    node: &TemplatePaneNodeData,
    menu_frame: &FrameRect,
    row_count: usize,
    row: usize,
) -> Option<FrameRect> {
    let row_height = popup_row_height(node, menu_frame, row_count)?;
    popup_row_frame(node, menu_frame, row_count, row, row_height)
}

pub(crate) fn menu_item_row_at_y(
    node: &TemplatePaneNodeData,
    menu_frame: &FrameRect,
    row_count: usize,
    y: f32,
) -> Option<usize> {
    let row_height = popup_row_height(node, menu_frame, row_count)?;
    let layout = popup_row_layout(node);
    let relative_y = y - menu_frame.y - layout.top;
    if !relative_y.is_finite() || relative_y < 0.0 {
        return None;
    }
    let stride = row_height + layout.spacing;
    let row = (relative_y / stride).floor() as usize;
    let frame = popup_row_frame(node, menu_frame, row_count, row, row_height)?;
    (y >= frame.y && y <= frame.y + frame.height).then_some(row)
}

pub(super) fn popup_row_frame(
    node: &TemplatePaneNodeData,
    popup_frame: &FrameRect,
    row_count: usize,
    row: usize,
    row_height: f32,
) -> Option<FrameRect> {
    if row >= row_count {
        return None;
    }
    let layout = popup_row_layout(node);
    let width = popup_frame.width - layout.left - layout.right;
    if !width.is_finite()
        || width <= 0.0
        || popup_row_height(node, popup_frame, row_count).is_none()
    {
        return None;
    }
    Some(FrameRect {
        x: popup_frame.x + layout.left,
        y: popup_frame.y + layout.top + row as f32 * (row_height + layout.spacing),
        width,
        height: row_height,
    })
}
