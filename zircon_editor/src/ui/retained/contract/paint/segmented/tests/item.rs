use super::*;

#[test]
fn collapsed_segment_has_no_divider_or_label_extent() {
    let segment = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 0.0,
        height: 0.0,
    };

    let divider = segment_divider_rect(&segment);
    let label = segment_label_rect(&segment);
    let allocated = segment_rect(&segment, 0, 0);

    assert_eq!((divider.width, divider.height), (0.0, 0.0));
    assert_eq!((label.width, label.height), (0.0, 0.0));
    assert_eq!((allocated.width, allocated.height), (0.0, 0.0));
}
