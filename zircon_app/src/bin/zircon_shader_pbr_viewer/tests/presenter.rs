use super::copy_rgba_to_xrgb;

#[test]
fn complete_rgba_frame_overwrites_the_surface_without_a_preclear() {
    let mut surface = [0x00ff_00ff, 0x00ff_00ff];

    let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255, 4, 5, 6, 255]);

    assert!(!cleared);
    assert_eq!(surface, [0x0001_0203, 0x0004_0506]);
}

#[test]
fn truncated_rgba_frame_clears_uncovered_surface_pixels() {
    let mut surface = [0x00ff_00ff, 0x00ff_00ff];

    let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255]);

    assert!(cleared);
    assert_eq!(surface, [0x0001_0203, 0]);
}
