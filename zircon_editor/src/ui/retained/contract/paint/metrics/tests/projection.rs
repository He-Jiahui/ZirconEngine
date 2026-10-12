use super::*;

#[test]
fn axis_label_metrics_project_from_host_control_metrics() {
    let mut host = current_host_metrics();
    host.font_body = 12.0;
    host.border_width = 1.5;
    host.line_height_ratio = 1.25;
    host.gap_m = 10.0;

    let metrics = axis_label_metrics_from_host(host);

    assert_eq!(metrics.font_size, 13.5);
    assert_eq!(metrics.line_height, 16.875);
    assert_eq!(metrics.link_lobe_width, 7.0);
    assert_eq!(metrics.link_lobe_height, 8.5);
    assert_eq!(metrics.link_lobe_radius, 3.5);
    assert_eq!(metrics.link_overlap, 3.0);
    assert_eq!(metrics.link_connector_width, 1.5);
}
