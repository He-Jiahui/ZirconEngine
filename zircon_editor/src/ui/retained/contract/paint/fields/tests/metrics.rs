use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn field_metrics_project_from_host_control_metrics() {
    let host_metrics = HostControlMetrics {
        row_height: 30.0,
        gap_s: 5.0,
        gap_m: 9.0,
        border_width: 2.0,
        font_body: 12.0,
        input_pad: [9.0, 9.0, 3.0, 5.0],
        ..METRICS
    };

    let metrics = workbench_field_metrics_from_host(host_metrics);

    assert_eq!(metrics.font_size, 12.0);
    assert!((metrics.line_height - 14.4).abs() < 0.001);
    assert_eq!(metrics.search_icon_size, 18.0);
    assert_eq!(metrics.search_text_left, 32.0);
    assert_eq!(metrics.search_max_height, 38.0);
    assert_eq!(metrics.stepper_width, 18.0);
    assert_eq!(metrics.stepper_divider_width, 2.0);
    assert_eq!(metrics.stepper_divider_inset_y, 5.0);
    assert_eq!(metrics.stepper_glyph_width, 13.0);
}

#[test]
fn authored_field_metrics_keep_zero_insets_and_scale_one_line_box() {
    let host = HostControlMetrics {
        scale_factor: 2.0,
        ..METRICS
    };
    let node = super::super::super::super::data::TemplatePaneNodeData {
        font_size: 17.0,
        line_height: 25.0,
        authored_field_padding: [Some(0.0), Some(7.0), None, None],
        ..Default::default()
    };
    let resolved = workbench_field_metrics_for_node_from_host(&node, host);
    let fallback = workbench_field_metrics_from_host(host);
    assert_eq!(resolved.font_size, 34.0);
    assert_eq!(resolved.line_height, 50.0);
    assert_eq!(resolved.input_pad_left, 0.0);
    assert_eq!(resolved.input_pad_right, 14.0);
    assert_eq!(
        resolved.search_text_left,
        fallback.search_text_left - fallback.input_pad_left
    );
    let missing = workbench_field_metrics_for_node_from_host(&Default::default(), host);
    assert_eq!(missing, fallback);
}

#[test]
fn authored_ratio_follows_size_and_explicit_pixels_take_priority() {
    let host = HostControlMetrics {
        scale_factor: 2.0,
        ..METRICS
    };
    let mut node = super::super::super::super::data::TemplatePaneNodeData {
        font_size: 17.0,
        line_height_ratio: 0.5,
        ..Default::default()
    };
    assert_eq!(
        workbench_field_metrics_for_node_from_host(&node, host).line_height,
        17.0
    );
    node.line_height = 12.0;
    assert_eq!(
        workbench_field_metrics_for_node_from_host(&node, host).line_height,
        24.0
    );
}

#[test]
fn authored_field_label_width_scales_once_without_resizing_plain_fields() {
    let host = HostControlMetrics {
        scale_factor: 1.5,
        ..METRICS
    };
    let mut node = super::super::super::super::data::TemplatePaneNodeData {
        authored_field_label_width: Some(92.0),
        ..Default::default()
    };
    assert_eq!(
        workbench_field_metrics_for_node_from_host(&node, host).label_width,
        138.0
    );
    for width in [Some(0.0), Some(-1.0), Some(f32::NAN), None] {
        node.authored_field_label_width = width;
        assert_eq!(
            workbench_field_metrics_for_node_from_host(&node, host).label_width,
            0.0
        );
    }
}
