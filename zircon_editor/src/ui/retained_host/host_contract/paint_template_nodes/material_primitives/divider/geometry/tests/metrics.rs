use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn divider_text_metrics_project_wrapper_and_centering_rules() {
    let rect = FrameRect {
        x: 0.0,
        y: 10.0,
        width: 160.0,
        height: 40.0,
    };

    let line_height = divider_label_line_height(12.0);
    let wrapped_width = divider_wrapped_label_width(32.0, 120.0);
    let metrics = divider_geometry_metrics_from_host(METRICS);

    assert!((line_height - METRICS.line_height(12.0)).abs() <= 0.01);
    assert!((wrapped_width - (32.0 + metrics.wrapper_horizontal_padding * 2.0)).abs() <= 0.01);
    assert!((divider_centered_label_y(&rect, line_height) - 22.8).abs() <= 0.01);
}

#[test]
fn divider_vertical_text_metrics_clamp_padding_and_min_extent() {
    assert!((divider_vertical_text_horizontal_padding(24.0) - 6.0).abs() <= 0.01);
    assert!((divider_min_text_frame_extent(0.2) - 1.0).abs() <= 0.01);
    assert!((divider_vertical_label_height(12.0, 28.0) - 28.0).abs() <= 0.01);
}

#[test]
fn divider_geometry_metrics_project_from_shared_host_metrics() {
    let host = HostControlMetrics {
        border_width: 1.5,
        font_small: 10.0,
        font_body: 15.0,
        line_height_ratio: 1.4,
        gap_m: 7.0,
        ..METRICS
    };

    let metrics = divider_geometry_metrics_from_host(host);

    assert_eq!(metrics.thickness, 1.5);
    assert_eq!(metrics.middle_horizontal_inset, 14.0);
    assert_eq!(metrics.inset_horizontal_inset, 63.0);
    assert_eq!(metrics.middle_vertical_inset, 7.0);
    assert_eq!(metrics.wrapper_horizontal_padding, 8.5);
    assert_eq!(metrics.wrapper_vertical_padding, 8.5);
    assert_eq!(metrics.default_font_size, 15.0);
    assert_eq!(metrics.minimum_font_size, 10.0);
    assert_eq!(metrics.line_height_ratio, 1.4);
    assert_eq!(metrics.minimum_text_frame_extent, 1.5);

    let authored = TemplatePaneNodeData {
        font_size: 17.5,
        ..TemplatePaneNodeData::default()
    };
    assert_eq!(
        divider_font_size_from_metrics(&TemplatePaneNodeData::default(), 40.0, metrics),
        15.0
    );
    assert_eq!(
        divider_font_size_from_metrics(&authored, 40.0, metrics),
        17.5
    );
}
