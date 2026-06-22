use super::super::super::super::data::FrameRect;
use super::super::geometry::pixel_aligned_rect;

#[test]
fn workbench_dropdown_preserves_half_pixel_declared_height() {
    let rect = pixel_aligned_rect(&FrameRect {
        x: 12.3,
        y: 8.4,
        width: 95.2,
        height: 30.5,
    });

    assert_eq!(rect.x, 12.0);
    assert_eq!(rect.y, 8.0);
    assert_eq!(rect.width, 95.0);
    assert_eq!(rect.height, 30.5);
}
