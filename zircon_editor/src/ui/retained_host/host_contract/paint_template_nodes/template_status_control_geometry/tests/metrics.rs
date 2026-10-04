use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn workbench_status_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.font_body = 11.0;
    host.font_large = 15.0;
    host.line_height_ratio = 1.25;
    host.radius_control = 3.0;
    host.border_width = 1.5;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.row_height = 28.0;

    let metrics = workbench_status_metrics_from_host(host);

    assert_eq!(metrics.font_size, 11.0);
    assert_eq!(metrics.line_height, 13.75);
    assert_eq!(metrics.radius, 3.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.text_inset, 5.0);
    assert_eq!(metrics.text_value_gap, 5.0);
    assert_eq!(metrics.icon_glyph_size, 15.0);
    assert_eq!(metrics.signal_icon_left, 9.0);
    assert_eq!(metrics.signal_text_gap, 9.0);
    assert_eq!(metrics.signal_marker_size, 9.0);
}
