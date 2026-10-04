use super::super::super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn table_action_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.border_width = 1.5;
    host.radius_control = 6.0;

    let metrics = table_action_metrics_from_host(host);

    assert_eq!(metrics.icon_size, 18.0);
    assert_eq!(metrics.button_size, 23.0);
    assert_eq!(metrics.action_column_width, 28.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.radius, 6.0);
}
