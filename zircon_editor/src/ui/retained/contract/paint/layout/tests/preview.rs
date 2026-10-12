use super::*;

fn metrics() -> DragOverlayMetrics {
    DragOverlayMetrics {
        border_width: 1.0,
        preview_radius: 4.0,
        icon_radius: 4.0,
        font_size: 13.33,
        line_height: 16.0,
        icon_left: 12.0,
        icon_size: 16.0,
        text_left_with_icon: 35.0,
        text_right_inset: 12.0,
        indicator_thickness: 2.0,
    }
}

#[test]
fn preview_content_stays_inside_a_narrow_short_drag_frame() {
    let preview = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 18.0,
        height: 8.0,
    };
    let metrics = metrics();

    assert_contained(preview_icon_frame(&preview, &metrics), &preview);
    assert_contained(preview_text_frame(&preview, &metrics), &preview);
}

#[test]
fn preview_frame_keeps_a_collapsed_fallback_collapsed() {
    let node = TemplatePaneNodeData::default();
    let fallback = FrameRect {
        x: 1.0,
        y: 2.0,
        width: 0.0,
        height: 0.0,
    };

    assert_eq!(preview_frame(&node, &fallback), fallback);
}

fn assert_contained(rect: FrameRect, parent: &FrameRect) {
    let epsilon = 0.000_1;
    assert!(rect.x >= parent.x - epsilon);
    assert!(rect.y >= parent.y - epsilon);
    assert!(rect.x + rect.width <= parent.x + parent.width + epsilon);
    assert!(rect.y + rect.height <= parent.y + parent.height + epsilon);
}
