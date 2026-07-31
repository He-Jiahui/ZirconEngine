use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::actions::table_action_column_width;
use super::super::identity::{is_table_header, is_table_tail};
use super::allocation::allocate_table_columns_for_node;
use super::metrics::table_cell_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cell_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let metrics = table_cell_metrics();
    let available_width =
        (rect.width - metrics.inset_x * 2.0 - table_action_column_width()).max(0.0);
    let columns = allocate_table_columns_for_node(node, available_width);
    let x = rect.x + metrics.inset_x + content_offset_x + columns.x_offset(index);
    let width = columns.width(index);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + metrics.inset_y + content_offset_y,
        width: width.max(0.0),
        height: (rect.height - metrics.inset_y * 2.0).max(0.0),
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
