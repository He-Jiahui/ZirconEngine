use super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn dropdown_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;
    host.radius_control = 6.0;
    host.font_body = 11.0;
    host.line_height_ratio = 1.4;
    host.input_pad = [9.0, 8.0, 5.0, 4.0];
    host.button_chevron_reserve = 24.0;
    host.button_icon_gap = 6.0;
    host.gap_s = 5.0;

    let metrics = workbench_dropdown_metrics_from_host(host);

    assert_eq!(metrics.border_width, 2.0);
    assert_eq!(metrics.radius, 6.0);
    assert_eq!(metrics.font_size, 11.0);
    assert!((metrics.line_height - 15.4).abs() < f32::EPSILON);
    assert_eq!(metrics.text_inset_x, 9.0);
    assert_eq!(metrics.chevron_size, 23.0);
    assert_eq!(metrics.chevron_right, 6.0);
    assert_eq!(metrics.chevron_reserve, 34.0);
}
