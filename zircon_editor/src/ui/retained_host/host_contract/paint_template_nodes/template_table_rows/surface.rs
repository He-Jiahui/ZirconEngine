//! 完整表格行绘制底面与未选中时的行分隔；尾行/旧选中行的声明偏移改变整体行框。
//! 单元格内容偏移另由 cells 处理，两者不能在各绘制分支重复累加。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_table_selected, is_table_tail};
use super::layers::separator_order;
use super::metrics::table_row_surface_metrics;
use super::style::{
    table_row_background, table_row_border, table_row_border_width, table_row_style,
};

const TABLE_ROW_SURFACE_COMMAND_CAPACITY: usize = 2;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    commands.reserve(TABLE_ROW_SURFACE_COMMAND_CAPACITY);
    let metrics = table_row_surface_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(table_row_background(node)),
        table_row_border(node),
        table_row_border_width(node),
        metrics.radius,
        opacity,
    ));
    if !is_selected_row(node) {
        let separator_height = metrics.separator_height.min(rect.height).max(0.0);
        let separator = FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - separator_height).max(0.0),
            width: rect.width,
            height: separator_height,
        };
        if intersect(&separator, clip).is_none() {
            return;
        }
        commands.push(HostPaintCommand::quad(
            separator,
            Some(clip.clone()),
            separator_order(order),
            Some(table_row_style(node).separator),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    if is_table_tail(node) || is_table_selected(node) {
        FrameRect {
            x: rect.x + node.layout_offset_x,
            y: rect.y + node.layout_offset_y,
            width: rect.width,
            height: rect.height,
        }
    } else {
        rect.clone()
    }
}

fn is_selected_row(node: &TemplatePaneNodeData) -> bool {
    node.selected || node.checked || is_table_selected(node)
}

#[cfg(test)]
#[path = "tests/surface_optimization_tests.rs"]
mod optimization_tests;
