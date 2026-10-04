use super::*;

#[test]
fn viewport_prop_corner_radius_stays_inside_a_narrow_surface() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.5,
        height: 24.0,
    };

    assert_eq!(template_corner_radius_from_rect(&rect), 0.25);
}
