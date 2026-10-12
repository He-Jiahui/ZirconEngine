use super::*;

#[test]
fn status_icon_centered_rect_clamps_to_the_available_extent() {
    let origin = FrameRect {
        x: 1.0,
        y: 3.0,
        width: 20.0,
        height: 24.0,
    };

    let rect = centered_rect(&origin, 28.0);

    assert_eq!(rect, origin);
}
