use super::*;

#[test]
fn invalid_centered_slider_geometry_has_no_drawable_extent() {
    let invalid = centered_rect(8.0, 6.0, f32::NAN);

    assert_eq!((invalid.width, invalid.height), (0.0, 0.0));
    assert_eq!((invalid.x, invalid.y), (8.0, 6.0));
}
