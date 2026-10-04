use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn compact_icon_text_uses_shared_caption_size_instead_of_instance_font_size() {
    let node = TemplatePaneNodeData {
        component_variant: "code compact_icon_text".into(),
        font_size: 24.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 54.0,
        height: 28.0,
        ..FrameRect::default()
    };

    let metrics = button_content_metrics_from_host(METRICS);
    assert_eq!(
        button_label_font_size_from_metrics(&node, &rect, metrics),
        METRICS.font_small
    );
}

#[test]
fn explicit_button_font_sizes_follow_host_scale_and_fallback_stays_physical() {
    let node = TemplatePaneNodeData {
        font_size: 12.0,
        ..TemplatePaneNodeData::default()
    };
    let fallback_node = TemplatePaneNodeData::default();
    let rect = FrameRect {
        width: 120.0,
        height: 40.0,
        ..FrameRect::default()
    };
    let high_dpi_metrics = HostControlMetrics {
        scale_factor: 1.5,
        font_small: METRICS.font_small * 1.5,
        font_body: METRICS.font_body * 1.5,
        ..METRICS
    };

    assert_eq!(
        button_label_font_size_from_metrics(
            &node,
            &rect,
            button_content_metrics_from_host(METRICS),
        ),
        12.0
    );
    let high_dpi_button_metrics = button_content_metrics_from_host(high_dpi_metrics);
    assert_eq!(
        button_label_font_size_from_metrics(&node, &rect, high_dpi_button_metrics),
        18.0
    );
    assert_eq!(
        button_label_font_size_from_metrics(&fallback_node, &rect, high_dpi_button_metrics),
        high_dpi_metrics.font_body,
        "host metric fallback is already physical and must not be scaled again"
    );
}

#[test]
fn button_label_font_size_shrinks_only_when_the_node_declares_shrink_overflow() {
    let node = TemplatePaneNodeData {
        overflow: "shrink".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 72.0,
        height: METRICS.control_default_height,
        ..FrameRect::default()
    };
    let label = "Disabled";
    let text_style = UiTextRunPaintStyle::default();
    let base_font_size = button_label_font_size(&node, &rect);
    let fitted_font_size =
        button_label_font_size_for_slot(&node, &rect, label, text_style, 0.0, 0.0);
    let available_ink_width =
        (max_label_slot_width(&node, &rect) - button_content_metrics().text_clip_guard).max(0.0);

    assert!(fitted_font_size < base_font_size);
    assert!(
        measured_label_ink_width(label, fitted_font_size, text_style) <= available_ink_width + 0.01
    );
}

#[test]
fn button_label_font_size_preserves_elide_overflow_at_the_declared_size() {
    let node = TemplatePaneNodeData {
        overflow: "elide".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        width: 72.0,
        height: METRICS.control_default_height,
        ..FrameRect::default()
    };
    let text_style = UiTextRunPaintStyle::default();

    assert_eq!(
        button_label_font_size_for_slot(&node, &rect, "Disabled", text_style, 0.0, 0.0,),
        button_label_font_size(&node, &rect),
    );
}
