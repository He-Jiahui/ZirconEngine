//! 整行共享一次列分配和文字度量快照，再依次产生各列框，避免每列重复计算布局。
//! 表头/尾行内容偏移和每列微调只影响单元格位置，不修改列可见性。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::actions::table_action_column_width;
use super::super::identity::{is_table_header, is_table_tail};
use super::allocation::{allocate_table_columns_for_node, TableColumnLayout};
use super::metrics::{table_cell_metrics, WorkbenchTableCellMetrics, TABLE_COLUMN_COUNT};

#[derive(Clone, Copy)]
struct TableCellLayout {
    content_offset_x: f32,
    content_offset_y: f32,
    metrics: WorkbenchTableCellMetrics,
    columns: TableColumnLayout,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cell_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
) -> FrameRect {
    let layout = table_cell_layout(node, rect);
    table_cell_rect_from_layout(node, rect, index, layout, layout.columns.x_offset(index))
}

pub(super) fn table_cell_rects(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> [FrameRect; TABLE_COLUMN_COUNT] {
    // Column minimums use Runtime Text measurement, so a row shares one allocation snapshot.
    let layout = table_cell_layout(node, rect);
    let mut x_offset = 0.0;
    std::array::from_fn(|index| {
        let cell = table_cell_rect_from_layout(node, rect, index, layout, x_offset);
        x_offset += layout.columns.width(index);
        cell
    })
}

fn table_cell_layout(node: &TemplatePaneNodeData, rect: &FrameRect) -> TableCellLayout {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let metrics = table_cell_metrics();
    let available_width =
        (rect.width - metrics.inset_x * 2.0 - table_action_column_width()).max(0.0);
    TableCellLayout {
        content_offset_x,
        content_offset_y,
        metrics,
        columns: allocate_table_columns_for_node(node, available_width),
    }
}

fn table_cell_rect_from_layout(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
    layout: TableCellLayout,
    x_offset: f32,
) -> FrameRect {
    let x = rect.x + layout.metrics.inset_x + layout.content_offset_x + x_offset;
    let width = layout.columns.width(index);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + layout.metrics.inset_y + layout.content_offset_y,
        width: width.max(0.0),
        height: (rect.height - layout.metrics.inset_y * 2.0).max(0.0),
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

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
