use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn alert_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;
    host.radius_control = 3.0;
    host.font_body = 12.5;
    host.line_height_ratio = 1.4;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 14.0;
    host.row_height = 34.0;

    let alert = alert_metrics_from_host(host);

    assert_eq!(alert.border_width, 2.0);
    assert_eq!(alert.radius, 8.0);
    assert_eq!(alert.font_size, 12.5);
    assert!((alert.line_height - 17.5).abs() < f32::EPSILON);
    assert_eq!(alert.icon_size, 21.0);
    assert_eq!(alert.icon_left, 10.0);
    assert_eq!(alert.text_gap, 9.0);
    assert_eq!(alert.text_vertical_inset, host.gap_s + host.border_width);
    assert_eq!(alert.text_right_inset, 10.0);
}

#[test]
fn toast_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;
    host.radius_control = 3.0;
    host.font_small = 11.0;
    host.line_height_ratio = 1.3;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.row_height = 32.0;

    let toast = toast_metrics_from_host(host);

    assert_eq!(toast.border_width, 2.0);
    assert_eq!(toast.radius, 8.0);
    assert_eq!(toast.font_size, 11.0);
    assert!((toast.line_height - 14.3).abs() < f32::EPSILON);
    assert_eq!(toast.icon_size, 19.0);
    assert_eq!(toast.icon_left, 13.0);
    assert_eq!(toast.text_gap, 11.0);
    assert_eq!(toast.trailing_inset, 9.0);
    assert_eq!(toast.close_size, 16.0);
    assert_eq!(toast.action_gap, 5.0);
    assert_eq!(toast.action_width, 46.0);
    assert_eq!(toast.action_minimum_width, 231.0);
}
