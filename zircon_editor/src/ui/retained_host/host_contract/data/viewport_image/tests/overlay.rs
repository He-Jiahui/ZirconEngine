use super::*;

#[test]
fn offscreen_line_is_clipped_before_overlay_allocation() {
    let overlay = HostViewportOverlayImageData::from_screen_lines(
        "clip-test",
        UVec2::new(320, 180),
        &[HandleScreenLine::new(
            Vec2::new(-10_000.0, 90.0),
            Vec2::new(20.0, 90.0),
            Vec4::ONE,
            2.0,
            None,
        )],
    )
    .expect("a partially visible line should rasterize");

    assert_eq!(overlay.x, 0);
    assert!(overlay.width < 32);
    assert!(overlay.height < 16);
}

#[test]
fn disjoint_line_does_not_allocate_an_overlay() {
    assert!(HostViewportOverlayImageData::from_screen_lines(
        "clip-test",
        UVec2::new(320, 180),
        &[HandleScreenLine::new(
            Vec2::new(-20.0, -20.0),
            Vec2::new(-10.0, -10.0),
            Vec4::ONE,
            2.0,
            None,
        )],
    )
    .is_none());
}

#[test]
fn source_over_preserves_transparency_and_blends_coverage_in_linear_light() {
    let mut transparent = [0, 0, 0, 0];
    blend_source_over(&mut transparent, [255, 255, 255, 128], 1.0);
    assert_eq!(transparent, [255, 255, 255, 128]);

    let mut opaque_black = [0, 0, 0, 255];
    blend_source_over(&mut opaque_black, [255, 255, 255, 255], 0.5);
    assert_eq!(opaque_black, [188, 188, 188, 255]);
}
