use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn template_node_font_role_projects_caption_and_body_from_host_typography() {
    let metrics = HostControlMetrics {
        font_small: 11.0,
        font_body: 15.0,
        ..METRICS
    };
    let body = TemplatePaneNodeData::default();
    let mut caption = TemplatePaneNodeData::default();
    caption.role = "Label".into();
    caption.text_tone = "muted".into();
    let mut authored = caption.clone();
    authored.font_size = 12.0;
    let mut subtle = caption.clone();
    subtle.text_tone = "subtle".into();
    let mut secondary = caption.clone();
    secondary.text_tone = "secondary".into();

    assert_eq!(node_font_size_from_host(&body, 20.0, metrics), 15.0);
    assert_eq!(node_font_size_from_host(&body, 10.0, metrics), 15.0);
    assert_eq!(node_font_size_from_host(&body, 0.0, metrics), 0.0);
    assert_eq!(node_font_size_from_host(&caption, 20.0, metrics), 11.0);
    assert_eq!(node_font_size_from_host(&caption, 6.0, metrics), 11.0);
    assert_eq!(node_font_size_from_host(&subtle, 20.0, metrics), 11.0);
    assert_eq!(node_font_size_from_host(&secondary, 20.0, metrics), 11.0);
    assert_eq!(node_font_size_from_host(&authored, 20.0, metrics), 12.0);
    assert_eq!(node_font_size_from_host(&authored, 6.0, metrics), 12.0);
}

#[test]
fn explicit_font_sizes_scale_from_logical_units_once_for_physical_paint() {
    let mut authored = TemplatePaneNodeData::default();
    authored.font_size = 12.0;
    let logical_metrics = HostControlMetrics {
        scale_factor: 1.0,
        ..METRICS
    };
    let high_dpi_metrics = HostControlMetrics {
        font_small: METRICS.font_small * 1.5,
        font_body: METRICS.font_body * 1.5,
        font_large: METRICS.font_large * 1.5,
        scale_factor: 1.5,
        ..METRICS
    };

    assert_eq!(
        node_font_size_from_host(&authored, 24.0, logical_metrics),
        12.0
    );
    assert_eq!(
        node_font_size_from_host(&authored, 36.0, high_dpi_metrics),
        18.0
    );
    assert!(
        (template_node_text_line_height_from_host(18.0, high_dpi_metrics) - 25.2).abs() < 0.001
    );

    let default_body = TemplatePaneNodeData::default();
    assert_eq!(
        node_font_size_from_host(&default_body, 36.0, high_dpi_metrics),
        21.0,
        "fallback metrics are already physical and must not be scaled twice"
    );
}

#[test]
fn template_node_text_geometry_projects_shared_gap_border_and_caption_line_height() {
    let metrics = HostControlMetrics {
        gap_s: 5.0,
        border_width: 1.5,
        font_small: 10.0,
        line_height_ratio: 1.3,
        ..METRICS
    };

    let geometry = template_node_text_geometry_metrics_from_host(metrics);

    assert_eq!(geometry.horizontal_inset, 5.0);
    assert_eq!(geometry.vertical_inset, 5.0);
    assert_eq!(geometry.minimum_text_height, 13.0);
    assert_eq!(geometry.edge_guard, 1.5);
}

#[test]
fn template_node_text_line_height_keeps_the_runtime_fractional_metric() {
    let metrics = HostControlMetrics {
        line_height_ratio: 1.35,
        ..METRICS
    };

    assert!((template_node_text_line_height_from_host(11.0, metrics) - 14.85).abs() < 0.001);
}
