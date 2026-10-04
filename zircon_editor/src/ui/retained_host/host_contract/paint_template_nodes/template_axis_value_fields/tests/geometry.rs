use super::*;

#[test]
fn axis_field_preserves_fractional_post_dpi_geometry() {
    let rect = axis_field_rect(&FrameRect {
        x: 12.25,
        y: 7.5,
        width: 81.75,
        height: 31.5,
    });

    assert_eq!(rect.x, 12.25);
    assert_eq!(rect.width, 81.75);
    assert!(rect.y.fract() != 0.0);
    assert!(rect.height > 0.0);
}
