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

#[test]
fn status_caption_uses_authored_size_ratio_and_single_dpi_scale() {
    let host = HostControlMetrics { scale_factor: 1.5, ..METRICS };
    let mut node = crate::ui::retained_host::host_contract::data::TemplatePaneNodeData {
        font_size: 10.666667, line_height_ratio: 1.2, ..Default::default()
    };
    let (font, line) = super::status_text_metrics_for_node_from_host(&node, host);
    assert!((font - 16.0).abs() < 0.0001);
    assert!((line - 19.2).abs() < 0.0001);
    node.line_height = 20.0;
    assert_eq!(super::status_text_metrics_for_node_from_host(&node, host).1, 30.0);
    let defaults = super::status_text_metrics_for_node_from_host(&Default::default(), host);
    assert_eq!(defaults, (host.font_body, host.line_height(host.font_body)));
}
