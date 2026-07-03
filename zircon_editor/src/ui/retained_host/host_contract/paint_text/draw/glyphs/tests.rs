use super::super::placement::RETAINED_TEXT_SUBPIXEL_BINS;
use super::*;

#[test]
fn retained_glyph_placement_uses_eight_phase_bins_for_editor_labels() {
    assert_eq!(TEXT_RASTER_SUPERSAMPLE, 8.0);
    assert_eq!(RETAINED_TEXT_SUBPIXEL_BINS, 8);
}

#[test]
fn logical_raster_extent_downsamples_scaled_bitmap_bounds() {
    assert_eq!(logical_raster_extent(0, 4.0, 0.0), 0);
    assert_eq!(logical_raster_extent(1, 4.0, 0.0), 1);
    assert_eq!(logical_raster_extent(3, 4.0, 0.0), 1);
    assert_eq!(logical_raster_extent(4, 4.0, 0.0), 1);
    assert_eq!(logical_raster_extent(5, 4.0, 0.0), 2);
}

#[test]
fn logical_raster_extent_keeps_native_subpixel_bitmap_bounds() {
    assert_eq!(logical_raster_extent(0, 1.0, 0.0), 0);
    assert_eq!(logical_raster_extent(1, 1.0, 0.0), 1);
    assert_eq!(logical_raster_extent(4, 1.0, 0.0), 4);
}

#[test]
fn logical_raster_extent_keeps_fractional_fallback_tail_pixel() {
    assert_eq!(logical_raster_extent(4, 4.0, 0.0), 1);
    assert_eq!(logical_raster_extent(4, 4.0, 0.5), 2);
}

#[test]
fn retained_glyph_placement_uses_stable_subpixel_bins_instead_of_rounding_each_glyph() {
    let left = RetainedGlyphPlacement::from_screen_x(20.10);
    let middle = RetainedGlyphPlacement::from_screen_x(20.45);
    let right = RetainedGlyphPlacement::from_screen_x(20.80);

    assert_eq!(left.pixel_x, 20);
    assert_eq!(middle.pixel_x, 20);
    assert_eq!(right.pixel_x, 20);
    assert!((left.subpixel_offset - 0.125).abs() < 0.001);
    assert!((middle.subpixel_offset - 0.5).abs() < 0.001);
    assert!((right.subpixel_offset - 0.75).abs() < 0.001);
}

#[test]
fn retained_glyph_placement_uses_nearest_subpixel_bin_without_left_bias() {
    let near_left = RetainedGlyphPlacement::from_screen_x(20.20);
    let half = RetainedGlyphPlacement::from_screen_x(20.50);
    let near_right = RetainedGlyphPlacement::from_screen_x(20.90);
    let next_pixel = RetainedGlyphPlacement::from_screen_x(20.95);

    assert_eq!(near_left.pixel_x, 20);
    assert!((near_left.subpixel_offset - 0.25).abs() < 0.001);
    assert_eq!(half.pixel_x, 20);
    assert!((half.subpixel_offset - 0.5).abs() < 0.001);
    assert_eq!(near_right.pixel_x, 20);
    assert!((near_right.subpixel_offset - 0.875).abs() < 0.001);
    assert_eq!(next_pixel.pixel_x, 21);
    assert_eq!(next_pixel.subpixel_offset, 0.0);
}
