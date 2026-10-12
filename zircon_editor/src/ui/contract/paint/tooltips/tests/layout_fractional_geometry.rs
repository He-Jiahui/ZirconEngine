use super::*;

#[test]
fn tooltip_paint_rect_preserves_fractional_post_dpi_geometry() {
    let rect = paint_rect(&FrameRect {
        x: 14.25,
        y: 19.5,
        width: 176.75,
        height: 64.25,
    });

    assert_eq!(rect.x, 14.25);
    assert_eq!(rect.y, 19.5);
    assert_eq!(rect.width, 176.75);
    assert_eq!(rect.height, 64.25);
}
