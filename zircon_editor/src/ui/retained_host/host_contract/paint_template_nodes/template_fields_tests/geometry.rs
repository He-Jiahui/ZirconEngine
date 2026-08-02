use super::super::super::super::data::FrameRect;
use super::super::geometry::{field_surface_radius, pixel_aligned_rect};

#[test]
fn workbench_field_alignment_does_not_expand_declared_bounds() {
    let rect = pixel_aligned_rect(&FrameRect {
        x: 12.3,
        y: 8.4,
        width: 67.2,
        height: 30.5,
    });

    assert_eq!(rect.x, 13.0);
    assert_eq!(rect.y, 9.0);
    assert_eq!(rect.width, 66.0);
    assert_eq!(rect.height, 29.0);
}

#[test]
fn workbench_field_surface_radius_stays_inside_a_narrow_control() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 32.0,
    };

    assert_eq!(field_surface_radius(&rect, 4.0), 0.25);
}
