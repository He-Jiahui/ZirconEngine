use super::*;

fn metrics() -> TemplateNodeTextGeometryMetrics {
    TemplateNodeTextGeometryMetrics {
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
