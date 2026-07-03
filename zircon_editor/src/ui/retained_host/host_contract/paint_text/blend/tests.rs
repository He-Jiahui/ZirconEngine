use super::*;

#[test]
fn alpha_text_blend_preserves_opaque_retained_frame() {
    let mut frame = HostRgbaFrame::filled(1, 1, [100, 120, 140, 255]);

    blend_pixel(&mut frame, 0, 0, [200, 40, 80, 128]);

    assert_eq!(&frame.as_bytes()[0..4], &[150, 79, 109, 255]);
}

#[test]
fn transparent_text_blend_leaves_pixel_untouched() {
    let mut frame = HostRgbaFrame::filled(1, 1, [100, 120, 140, 255]);

    blend_pixel(&mut frame, 0, 0, [200, 40, 80, 0]);

    assert_eq!(&frame.as_bytes()[0..4], &[100, 120, 140, 255]);
}

#[test]
fn subpixel_text_blend_uses_independent_rgb_coverage() {
    let mut frame = HostRgbaFrame::filled(1, 1, [100, 120, 140, 255]);

    blend_pixel_channel_coverage(&mut frame, 0, 0, [200, 40, 80, 255], [255, 0, 128]);

    assert_eq!(&frame.as_bytes()[0..4], &[200, 120, 109, 255]);
}

#[test]
fn subpixel_text_blend_multiplies_color_alpha_into_coverage() {
    let mut frame = HostRgbaFrame::filled(1, 1, [100, 120, 140, 255]);

    blend_pixel_channel_coverage(&mut frame, 0, 0, [200, 40, 80, 128], [255, 255, 255]);

    assert_eq!(&frame.as_bytes()[0..4], &[150, 79, 109, 255]);
}
