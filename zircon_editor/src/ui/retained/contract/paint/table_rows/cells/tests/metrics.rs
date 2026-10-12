use super::super::super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn table_cell_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.font_body = 11.0;
    host.line_height_ratio = 1.35;
    host.gap_m = 9.0;
    host.gap_s = 5.0;
    host.text_clip_guard = 7.0;

    let metrics = table_cell_metrics_from_host(host);

    assert_eq!(metrics.font_size, 11.0);
    assert!((metrics.line_height - 14.85).abs() < 0.0001);
    assert_eq!(metrics.inset_x, 9.0);
    assert_eq!(metrics.inset_y, 5.0);
    assert_eq!(metrics.text_clip_guard, 7.0);
}

#[test]
fn table_column_metrics_project_readable_minimums_from_host_control_metrics() {
    let mut host = METRICS;
    host.font_body = 0.0;
    host.row_height = 30.0;
    host.gap_m = 10.0;
    host.text_clip_guard = 4.0;

    let metrics = table_column_metrics_from_host(host);

    assert_eq!(metrics.ratios, [0.36, 0.27, 0.19, 0.18]);
    assert_eq!(metrics.min_widths, [130.0, 60.0, 60.0, 76.0]);
    assert_eq!(metrics.drop_order, [3, 2, 1, 0]);
}

#[test]
fn table_column_minimums_include_runtime_text_measurement() {
    let mut host = METRICS;
    host.font_body = 12.0;
    host.row_height = 1.0;
    host.gap_m = 3.0;
    host.text_clip_guard = 2.0;

    let metrics = table_column_metrics_from_host(host);
    let expected_revision_min =
        measure_runtime_text_width(REVISION_COLUMN_WIDTH_SAMPLE, 12.0) + 8.0;

    assert!(metrics.min_widths[3] >= expected_revision_min);
}
