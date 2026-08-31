use super::super::placement::{
    retained_glyph_placement_for_smoothing, RetainedGlyphPlacement, RETAINED_TEXT_SUBPIXEL_BINS,
};
use super::*;
use crate::ui::retained_host::host_contract::paint_theme::HostTextSmoothing;

#[test]
fn retained_glyph_placement_uses_eight_phase_bins_for_editor_labels() {
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
fn logical_raster_extent_keeps_combined_sample_offset_over_one_pixel() {
    assert_eq!(logical_raster_extent(16, 8.0, 1.125), 4);
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

#[test]
fn retained_glyph_bitmap_pixel_x_uses_raster_bearing_from_layout_pen_origin() {
    let glyph = RuntimeTextGlyph {
        glyph_index: 1,
        px: 13.0,
        x: 19.875,
        origin_x: 20.375,
        y: 4.0,
        raster_font_index: None,
    };
    let layout_bitmap_left_pixel_x = RetainedGlyphPlacement::from_screen_x(glyph.x).pixel_x;
    let origin_pixel_x = RetainedGlyphPlacement::from_screen_x(glyph.origin_x).pixel_x;

    assert_eq!(
        retained_glyph_bitmap_pixel_x(&glyph, layout_bitmap_left_pixel_x, origin_pixel_x, 2),
        origin_pixel_x + 2,
        "editor-label draw placement must keep the layout pen origin but use the actual raster bearing for the bitmap-left edge"
    );
}

#[test]
fn retained_glyph_bitmap_pixel_x_ignores_stale_layout_left_when_origin_is_valid() {
    let glyph = RuntimeTextGlyph {
        glyph_index: 1,
        px: 13.0,
        x: 15.125,
        origin_x: 20.875,
        y: 4.0,
        raster_font_index: None,
    };
    let layout_bitmap_left_pixel_x = RetainedGlyphPlacement::from_screen_x(glyph.x).pixel_x;
    let origin_pixel_x = RetainedGlyphPlacement::from_screen_x(glyph.origin_x).pixel_x;

    assert_eq!(
        retained_glyph_bitmap_pixel_x(&glyph, layout_bitmap_left_pixel_x, origin_pixel_x, -1),
        origin_pixel_x - 1,
        "a valid pen origin must keep raster placement stable even when layout bitmap-left was computed from a different bearing"
    );
}

#[test]
fn retained_glyph_bitmap_pixel_x_falls_back_to_layout_bitmap_left_for_invalid_origin() {
    let glyph = RuntimeTextGlyph {
        glyph_index: 1,
        px: 13.0,
        x: 20.51,
        origin_x: f32::NAN,
        y: 4.0,
        raster_font_index: None,
    };
    let grayscale_placement =
        retained_glyph_placement_for_smoothing(glyph.x, HostTextSmoothing::Grayscale);
    let subpixel_placement =
        retained_glyph_placement_for_smoothing(glyph.x, HostTextSmoothing::Subpixel);

    assert_eq!(
        retained_glyph_bitmap_pixel_x(
            &glyph,
            grayscale_placement.pixel_x,
            grayscale_placement.pixel_x,
            -1,
        ),
        20,
        "invalid-origin grayscale fallback must keep the retained 8-bin bitmap-left pixel after line-origin snapping"
    );
    assert_eq!(
        retained_glyph_bitmap_pixel_x(
            &glyph,
            subpixel_placement.pixel_x,
            subpixel_placement.pixel_x,
            -1,
        ),
        20,
        "invalid-origin subpixel fallback keeps the retained 8-bin bitmap-left pixel"
    );
}

#[test]
fn retained_glyph_bitmap_pixel_x_falls_back_to_raster_bearing_for_invalid_layout_x() {
    let glyph = RuntimeTextGlyph {
        glyph_index: 1,
        px: 13.0,
        x: f32::NAN,
        origin_x: 20.375,
        y: 4.0,
        raster_font_index: None,
    };
    let origin_pixel_x = RetainedGlyphPlacement::from_screen_x(glyph.origin_x).pixel_x;

    assert_eq!(
        retained_glyph_bitmap_pixel_x(&glyph, 0, origin_pixel_x, -1),
        origin_pixel_x - 1,
        "invalid layout bitmap-left falls back to the raster bearing relative to the retained pen-origin phase"
    );
}
