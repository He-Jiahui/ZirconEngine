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
