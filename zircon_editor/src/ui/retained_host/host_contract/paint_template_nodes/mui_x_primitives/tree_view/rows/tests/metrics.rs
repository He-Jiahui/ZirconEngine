use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn tree_view_metrics_follow_the_workbench_density_baseline() {
    let metrics = tree_view_row_metrics_from_host(METRICS);

    assert_eq!(metrics.horizontal_inset, 4.0);
    assert_eq!(metrics.indent_step, 6.0);
    assert_eq!(metrics.row_gap, 1.0);
    assert_eq!(metrics.row_radius, 4.0);
    assert_eq!(metrics.marker_inset, 3.0);
    assert_eq!(metrics.marker_min_edge, 3.0);
    assert_eq!(metrics.marker_max_edge, 6.0);
}

#[test]
fn tree_view_metrics_reflow_from_compact_host_density() {
    let mut host = METRICS;
    host.gap_s = 3.0;
    host.gap_m = 6.0;
    host.border_width = 1.5;
    host.radius_control = 3.0;
    let metrics = tree_view_row_metrics_from_host(host);

    assert_eq!(metrics.horizontal_inset, 3.0);
    assert_eq!(metrics.indent_step, 3.0);
    assert_eq!(metrics.row_gap, 1.5);
    assert_eq!(metrics.row_radius, 3.0);
    assert_eq!(metrics.marker_inset, 1.5);
    assert_eq!(metrics.marker_min_edge, 1.5);
    assert_eq!(metrics.marker_max_edge, 3.0);
}
