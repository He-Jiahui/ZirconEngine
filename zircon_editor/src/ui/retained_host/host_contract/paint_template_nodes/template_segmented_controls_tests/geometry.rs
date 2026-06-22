use super::super::super::super::data::FrameRect;
use super::super::super::template_segmented_control_geometry::{segment_rect, segmented_body_rect};
use super::support::{frame_rect, labeled_segmented_node};

#[test]
fn segment_rects_split_available_width_evenly() {
    let rect = FrameRect {
        x: 6.0,
        y: 4.0,
        width: 150.0,
        height: 30.0,
    };

    assert_eq!(segment_rect(&rect, 0, 3).x, 6.0);
    assert_eq!(segment_rect(&rect, 1, 3).x, 56.0);
    assert_eq!(segment_rect(&rect, 2, 3).width, 50.0);
}

#[test]
fn segmented_control_offsets_group_label_body() {
    let node = labeled_segmented_node();
    let body = segmented_body_rect(&node, &frame_rect(&node.frame));

    assert_eq!(body.x, 18.0);
    assert_eq!(body.y, 22.0);
    assert_eq!(body.height, 30.0);
}
