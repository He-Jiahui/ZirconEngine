use std::sync::Arc;

use super::*;

#[test]
fn retained_text_raster_uses_swash_for_ui_face() {
    let glyph_index = font_for_face(HostTextFontFace::Ui)
        .expect("ui retained-host font")
        .lookup_glyph_index('P');
    let raster = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 3.0, 0.0);

    assert_eq!(raster.source, CachedGlyphRasterSource::Swash);
    assert!(
        matches!(
            raster.format,
            CachedGlyphRasterFormat::AlphaMask | CachedGlyphRasterFormat::SubpixelMask
        ),
        "swash may return either alpha or subpixel coverage depending on the font backend"
    );
    assert_eq!(raster.raster_scale, 1.0);
    assert!(raster.metrics.width > 0);
    assert!(raster.metrics.height > 0);
    assert!(raster.bitmap.iter().any(|coverage| *coverage > 0));
}

#[test]
fn swash_subpixel_mask_preserves_rgb_coverage_for_retained_host() {
    let (format, bitmap) = swash_bitmap(
        Content::SubpixelMask,
        vec![0, 255, 0, 0, 90, 120, 150, 0],
        2,
        1,
        HostTextSmoothing::Subpixel,
    )
    .expect("subpixel mask should convert");

    assert_eq!(format, CachedGlyphRasterFormat::SubpixelMask);
    assert_eq!(bitmap, vec![0, 255, 0, 0, 90, 120, 150, 0]);
}

#[test]
fn swash_subpixel_mask_can_follow_grayscale_smoothing_preference() {
    let (format, bitmap) = swash_bitmap(
        Content::SubpixelMask,
        vec![0, 255, 0, 0, 90, 120, 150, 0],
        2,
        1,
        HostTextSmoothing::Grayscale,
    )
    .expect("subpixel mask should convert");

    assert_eq!(format, CachedGlyphRasterFormat::AlphaMask);
    assert_eq!(bitmap, vec![255, 150]);
}

#[test]
fn swash_render_format_tracks_text_smoothing_preference() {
    assert_eq!(
        swash_format_for_smoothing(HostTextSmoothing::Grayscale),
        Format::Alpha
    );
    assert_eq!(
        swash_format_for_smoothing(HostTextSmoothing::Subpixel),
        Format::Subpixel
    );
}

#[test]
fn retained_text_raster_cache_separates_subpixel_bins_for_same_glyph() {
    let glyph_index = font_for_face(HostTextFontFace::Ui)
        .expect("ui retained-host font")
        .lookup_glyph_index('e');

    let left = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 3.0, 0.0);
    let right = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 3.0, 2.0 / 3.0);

    assert!(!Arc::ptr_eq(&left.bitmap, &right.bitmap));
    assert_eq!(left.source, CachedGlyphRasterSource::Swash);
    assert_eq!(right.source, CachedGlyphRasterSource::Swash);
}

#[test]
fn retained_text_raster_normalizes_invalid_subpixel_offsets() {
    assert_eq!(normalized_subpixel_offset(f32::NAN), 0.0);
    assert_eq!(normalized_subpixel_offset(-1.0), 0.0);
    assert_eq!(normalized_subpixel_offset(2.0), 0.999);
}

#[test]
fn swash_metrics_x_offset_is_relative_to_pen_origin() {
    let glyph_index = font_for_face(HostTextFontFace::Ui)
        .expect("ui retained-host font")
        .lookup_glyph_index('j');
    let metrics = swash_metrics(HostTextFontFace::Ui, glyph_index, 13.0, 1.0, 7, 11, -2, 9);

    assert_eq!(metrics.x_offset, -2);
}

#[test]
fn fontdue_fallback_metrics_x_offset_is_relative_to_pen_origin() {
    let font = font_for_face(HostTextFontFace::Ui).expect("ui retained-host font");
    let glyph_index = font.lookup_glyph_index('j');
    let logical_metrics = font.metrics_indexed(glyph_index, 13.0);

    let raster = rasterize_fontdue_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 3.0, 0.5);

    assert_eq!(
        raster.metrics.x_offset,
        logical_metrics.bounds.xmin.floor() as i32
    );
    assert_eq!(raster.sample_offset_x, 0.5);
}

#[test]
fn fontdue_fallback_returns_empty_raster_when_font_is_unavailable() {
    let raster = empty_fontdue_raster(4.0, 0.5);

    assert_eq!(raster.metrics, CachedGlyphMetrics::default());
    assert!(raster.bitmap.is_empty());
    assert_eq!(raster.source, CachedGlyphRasterSource::FontdueFallback);
    assert_eq!(raster.sample_offset_x, 0.5);
}
