use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::{is_table_header, is_table_tail};
use super::metrics::{
    TABLE_ACTION_WIDTH, TABLE_CELL_INSET_X, TABLE_CELL_INSET_Y, TABLE_COLUMN_RATIOS,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cell_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let mut x = rect.x + TABLE_CELL_INSET_X + content_offset_x;
    let available_width = (rect.width - TABLE_CELL_INSET_X * 2.0 - TABLE_ACTION_WIDTH).max(1.0);
    for ratio in TABLE_COLUMN_RATIOS.iter().take(index) {
        x += available_width * ratio;
    }
    let width = TABLE_COLUMN_RATIOS
        .get(index)
        .map(|ratio| available_width * ratio)
        .unwrap_or(available_width)
        .max(1.0);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + TABLE_CELL_INSET_Y + content_offset_y,
        width: width.max(1.0),
        height: (rect.height - TABLE_CELL_INSET_Y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_content_offset(
    node: &TemplatePaneNodeData,
) -> (f32, f32) {
    if is_table_header(node) || is_table_tail(node) {
        (node.layout_content_offset_x, node.layout_content_offset_y)
    } else {
        (0.0, 0.0)
    }
}

fn table_cell_offset_x(node: &TemplatePaneNodeData, index: usize) -> f32 {
    match index {
        0 => node.layout_first_cell_offset_x,
        1 => node.layout_second_cell_offset_x,
        2 => node.layout_third_cell_offset_x,
        3 => node.layout_fourth_cell_offset_x,
        _ => 0.0,
    }
}
