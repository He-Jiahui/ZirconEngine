use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn default_metrics_follow_the_workbench_row_density() {
    let row = hierarchy_row_metrics_from_host_metrics(METRICS);

    assert_eq!(row.row_x, 8.0);
    assert_eq!(row.row_y, 8.0);
    assert_eq!(row.row_height, 26.0);
    assert_eq!(row.row_gap, 1.0);
    assert_eq!(row.row_width_inset, 16.0);
    assert_eq!(hierarchy_row_y(row, 2, 3.0), 59.0);
    assert_eq!(hierarchy_row_width(120.0, row), 104.0);
    assert_eq!(hierarchy_content_height(3, row), 96.0);
}

#[test]
fn compact_density_reflows_all_shared_row_geometry() {
    let mut metrics = METRICS;
    metrics.gap_m = 6.0;
    metrics.border_width = 1.5;
    metrics.row_height = 20.0;
    metrics.font_body = 12.0;
    metrics.line_height_ratio = 1.25;
    let row = hierarchy_row_metrics_from_host_metrics(metrics);

    assert_eq!(row.row_x, 6.0);
    assert_eq!(row.row_y, 6.0);
    assert_eq!(row.row_height, 17.0);
    assert_eq!(row.row_gap, 1.5);
    assert_eq!(row.row_width_inset, 12.0);
    assert_eq!(hierarchy_row_y(row, 1, 2.0), 22.5);
    assert_eq!(hierarchy_row_width(10.0, row), 0.0);
    assert_eq!(hierarchy_content_height(2, row), 47.5);
}
