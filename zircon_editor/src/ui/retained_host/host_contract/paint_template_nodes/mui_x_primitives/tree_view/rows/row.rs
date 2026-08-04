use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::{tree_view_marker_color, tree_view_row_color};
use super::metrics::TreeViewRowMetrics;

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
mod tests {
    use super::*;

    const METRICS: TreeViewRowMetrics = TreeViewRowMetrics {
        horizontal_inset: 4.0,
        indent_step: 6.0,
        row_gap: 1.0,
        row_radius: 4.0,
        marker_inset: 3.0,
        marker_min_edge: 3.0,
        marker_max_edge: 6.0,
    };

    #[test]
    fn tree_view_rows_skip_collapsed_width_without_overflowing() {
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 8.0,
            height: 30.0,
        };

        assert!(tree_view_row_frame(&rect, METRICS, 7.0, 0).is_none());
    }

    #[test]
    fn tree_view_marker_stays_inside_tiny_row_bounds() {
        let row = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 4.0,
            height: 2.0,
        };
        let marker = tree_view_marker_frame(&row, METRICS).expect("tiny visible row has a marker");

        assert!(marker.x >= row.x);
        assert!(marker.right() <= row.right());
        assert!(marker.y >= row.y);
        assert!(marker.bottom() <= row.bottom());
    }
}
