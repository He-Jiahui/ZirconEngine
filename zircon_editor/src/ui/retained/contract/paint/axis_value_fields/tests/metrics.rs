use super::*;

#[test]
fn axis_value_field_metrics_project_from_host_control_metrics() {
    let mut host = current_host_metrics();
    host.row_height = 28.0;
    host.border_width = 1.5;
    host.radius_control = 3.0;
    host.font_body = 11.0;
    host.line_height_ratio = 1.25;
    host.input_pad[0] = 6.0;

    let metrics = axis_value_field_metrics_from_host(host);

    assert_eq!(metrics.max_height, 31.0);
    assert_eq!(metrics.radius, 3.0);
    assert_eq!(metrics.font_size, 11.0);
    assert_eq!(metrics.line_height, 13.75);
    assert_eq!(metrics.text_inset_x, 6.0);
}
