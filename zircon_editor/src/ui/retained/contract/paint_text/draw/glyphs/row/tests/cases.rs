use super::*;

#[test]
fn alpha_receipt_pixels_use_the_text_color_coverage() {
    assert_eq!(
        glyph_pixel(&[0, 128], 2, TextGlyphBitmapFormat::AlphaMask, 0, 0),
        GlyphPixel::Empty
    );
    assert_eq!(
        glyph_pixel(&[0, 128], 2, TextGlyphBitmapFormat::AlphaMask, 1, 0),
        GlyphPixel::Alpha(128)
    );
}

#[test]
fn subpixel_receipt_pixels_preserve_rgb_coverage() {
    assert_eq!(
        glyph_pixel(
            &[12, 34, 56, 0],
            1,
            TextGlyphBitmapFormat::SubpixelMask,
            0,
            0,
        ),
        GlyphPixel::Subpixel([12, 34, 56])
    );
}

#[test]
fn color_receipt_pixels_preserve_straight_rgba() {
    assert_eq!(
        glyph_pixel(
            &[90, 120, 150, 255],
            1,
            TextGlyphBitmapFormat::ColorRgba,
            0,
            0,
        ),
        GlyphPixel::Color([90, 120, 150, 255])
    );
}

#[test]
fn truncated_receipt_pixels_fail_closed() {
    assert_eq!(
        glyph_pixel(&[1, 2, 3], 1, TextGlyphBitmapFormat::ColorRgba, 0, 0,),
        GlyphPixel::Empty
    );
}

#[test]
fn colored_glyph_pixels_apply_run_opacity_without_tinting_rgb() {
    let mut frame = HostRgbaFrame::filled(1, 1, [0, 0, 0, 255]);
    GlyphPixel::Color([100, 150, 200, 255]).blend(&mut frame, 0, 0, [7, 8, 9, 128]);
    assert_eq!(frame.as_bytes(), &[50, 75, 100, 255]);
}
