use super::*;

#[test]
fn group_label_stays_inside_a_collapsed_or_invalid_segmented_frame() {
    let label = segmented_group_label_rect(&FrameRect {
        x: 12.0,
        y: 8.0,
        width: f32::NAN,
        height: 0.0,
    });

    assert_eq!(label.x, 12.0);
    assert_eq!(label.y, 8.0);
    assert_eq!((label.width, label.height), (0.0, 0.0));
}
