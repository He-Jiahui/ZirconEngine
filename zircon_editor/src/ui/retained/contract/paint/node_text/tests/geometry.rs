use super::*;

fn metrics() -> TemplateNodeTextGeometryMetrics {
    TemplateNodeTextGeometryMetrics {
        scale_factor: 1.0,
        horizontal_inset: 4.0,
        vertical_inset: 4.0,
        minimum_text_height: 13.0,
        edge_guard: 1.0,
    }
}

#[test]
fn template_node_text_rect_uses_relative_shared_insets() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 30.0,
    };

    let text = text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

    assert_eq!(
        text,
        FrameRect {
            x: 14.0,
            y: 24.0,
            width: 92.0,
            height: 22.0,
        }
    );
}

#[test]
fn template_node_text_rect_preserves_caption_slot_in_short_rows() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 14.0,
    };

    let text = text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

    assert_eq!(text.y, 1.0);
    assert_eq!(text.height, 12.0);
    assert!(text.x >= rect.x);
    assert!(text.x + text.width <= rect.x + rect.width);
}

#[test]
fn template_node_text_rect_rejects_non_finite_or_empty_parent_geometry() {
    let rect = FrameRect {
        x: f32::NAN,
        y: f32::INFINITY,
        width: -4.0,
        height: f32::NAN,
    };

    let text = text_rect_for_node_with_metrics(&TemplatePaneNodeData::default(), &rect, metrics());

    assert_eq!(text.x, 0.0);
    assert_eq!(text.y, 0.0);
    assert_eq!(text.width, 0.0);
    assert_eq!(text.height, 0.0);
}

#[test]
fn label_text_lane_preserves_arranged_width_without_implicit_control_padding() {
    let node = TemplatePaneNodeData {
        role: "Label".into(),
        text: "Layer mask".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 72.0,
        height: 20.0,
    };

    let text = text_rect_for_node_with_metrics(&node, &rect, metrics());

    assert_eq!(text.x, rect.x);
    assert_eq!(text.width, 72.0);
    let button = TemplatePaneNodeData {
        role: "Button".into(),
        ..node
    };
    let button_text = text_rect_for_node_with_metrics(&button, &rect, metrics());
    assert_eq!(button_text.x, rect.x + 4.0);
    assert_eq!(button_text.width, 64.0);
}

#[test]
fn label_authored_padding_scales_once_and_preserves_explicit_zero() {
    let mut node = TemplatePaneNodeData {
        role: "Label".into(),
        authored_field_padding: [Some(0.0), Some(7.0), None, None],
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 12.5,
        y: 25.0,
        width: 90.0,
        height: 25.0,
    };
    let high_dpi = TemplateNodeTextGeometryMetrics {
        scale_factor: 1.25,
        ..metrics()
    };
    let text = text_rect_for_node_with_metrics(&node, &rect, high_dpi);
    assert_eq!(text.x, 12.5);
    assert_eq!(text.width, 81.25);

    node.authored_field_padding = [Some(3.0), Some(7.0), None, None];
    let text = text_rect_for_node_with_metrics(&node, &rect, high_dpi);
    assert_eq!(text.x, 16.25);
    assert_eq!(text.width, 77.5);

    node.authored_field_padding = [Some(f32::NAN), Some(-1.0), None, None];
    let text = text_rect_for_node_with_metrics(&node, &rect, high_dpi);
    assert_eq!(text.x, rect.x);
    assert_eq!(text.width, rect.width);

    node.authored_field_padding = [Some(100.0), Some(7.0), None, None];
    let text = text_rect_for_node_with_metrics(&node, &rect, high_dpi);
    assert_eq!(text.x, rect.x + rect.width);
    assert_eq!(text.width, 0.0);
}
