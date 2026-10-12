use super::*;

#[test]
fn collapsed_selected_segment_has_no_indicator_extent() {
    let segment = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 0.0,
        height: 0.0,
    };

    let selected = selected_segment_rect(&segment);
    let underline = selected_segment_underline_rect(&selected, 3.0);

    assert_eq!((selected.width, selected.height), (0.0, 0.0));
    assert_eq!((underline.width, underline.height), (0.0, 0.0));
}
