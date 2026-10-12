use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn workbench_chip_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.font_body = 11.0;
    host.line_height_ratio = 1.25;
    host.radius_control = 3.0;
    host.border_width = 1.5;
    host.gap_m = 9.0;

    let metrics = workbench_chip_metrics_from_host(host);

    assert_eq!(metrics.font_size, 11.0);
    assert_eq!(metrics.line_height, 13.75);
    assert_eq!(metrics.radius, 3.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.text_left, 12.0);
    assert_eq!(metrics.text_right, 9.0);
    assert_eq!(metrics.chevron_size, 14.0);
    assert_eq!(metrics.chevron_right, 9.0);
    assert_eq!(metrics.chevron_reserve, 27.0);
    assert_eq!(
        metrics.chevron_reserve,
        metrics.chevron_size + metrics.chevron_right + host.gap_s
    );
}
