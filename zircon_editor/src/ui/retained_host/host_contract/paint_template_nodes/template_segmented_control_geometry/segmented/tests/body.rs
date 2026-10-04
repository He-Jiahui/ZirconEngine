use super::*;

#[test]
fn collapsed_segmented_body_has_no_drawable_extent() {
    let body = segmented_body_rect(
        &TemplatePaneNodeData::default(),
        &FrameRect {
            x: 12.0,
            y: 8.0,
            width: 0.0,
            height: 0.0,
        },
    );

    assert_eq!(body.width, 0.0);
    assert_eq!(body.height, 0.0);
}

#[test]
fn labeled_segmented_body_stays_inside_a_short_parent_frame() {
    let node = TemplatePaneNodeData {
        label_text: "Render mode".to_string(),
        ..TemplatePaneNodeData::default()
    };
    let body = segmented_body_rect(
        &node,
        &FrameRect {
            x: 12.0,
            y: 8.0,
            width: 200.0,
            height: 10.0,
        },
    );

    assert_eq!((body.x, body.y), (12.0, 18.0));
    assert_eq!((body.width, body.height), (200.0, 0.0));
}
