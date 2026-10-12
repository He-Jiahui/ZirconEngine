use super::*;

#[test]
fn command_palette_preserves_fractional_post_dpi_geometry() {
    let rect = paint_rect(&FrameRect {
        x: 11.25,
        y: 17.5,
        width: 319.75,
        height: 241.25,
    });

    assert_eq!(rect.x, 11.25);
    assert_eq!(rect.y, 17.5);
    assert_eq!(rect.width, 319.75);
    assert_eq!(rect.height, 241.25);
}
