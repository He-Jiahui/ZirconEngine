use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{tree_view_marker_color, tree_view_row_color};
use super::metrics::TreeViewRowMetrics;

/// 标记几何保持在行框内，并仍受节点裁剪；选中或勾选控制首行背景，展开或弹层状态控制标记与次行悬停色。
pub(super) fn push_tree_view_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: TreeViewRowMetrics,
    row_height: f32,
    row: i32,
) {
    let Some(row_rect) = tree_view_row_frame(rect, metrics, row_height, row) else {
        return;
    };
    super::super::super::push_quad(
        commands,
        row_rect.clone(),
        clip,
        order + 1 + row,
        tree_view_row_color(node, row),
        0.0,
        metrics
            .row_radius
            .min(row_rect.width.min(row_rect.height) * 0.5),
        opacity,
    );
    let Some(marker_rect) = tree_view_marker_frame(&row_rect, metrics) else {
        return;
    };
    super::super::super::push_quad(
        commands,
        marker_rect.clone(),
        clip,
        order + 5 + row,
        tree_view_marker_color(node, row),
        0.0,
        marker_rect.width * 0.5,
        opacity,
    );
}

fn tree_view_row_frame(
    rect: &FrameRect,
    metrics: TreeViewRowMetrics,
    row_height: f32,
    row: i32,
) -> Option<FrameRect> {
    let row_y = rect.y + metrics.horizontal_inset + row as f32 * row_height;
    let row_indent = row as f32 * metrics.indent_step;
    let row_rect = FrameRect {
        x: rect.x + metrics.horizontal_inset + row_indent,
        y: row_y,
        width: (rect.width - metrics.horizontal_inset * 2.0 - row_indent).max(0.0),
        height: (row_height - metrics.row_gap).max(0.0),
    };
    (row_rect.width > 0.0 && row_rect.height > 0.0).then_some(row_rect)
}

fn tree_view_marker_frame(row_rect: &FrameRect, metrics: TreeViewRowMetrics) -> Option<FrameRect> {
    let marker_size = (row_rect.height * 0.45)
        .max(metrics.marker_min_edge)
        .min(metrics.marker_max_edge)
        .min(row_rect.width.min(row_rect.height));
    if marker_size <= 0.0 {
        return None;
    }

    Some(FrameRect {
        x: (row_rect.x + metrics.marker_inset).min(row_rect.x + row_rect.width - marker_size),
        y: row_rect.y + (row_rect.height - marker_size) * 0.5,
        width: marker_size,
        height: marker_size,
    })
}

#[cfg(test)]
#[path = "tests/row.rs"]
mod tests;
