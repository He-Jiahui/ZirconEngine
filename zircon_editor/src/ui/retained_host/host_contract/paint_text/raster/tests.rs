use std::sync::Arc;

use super::*;

#[test]
fn retained_text_raster_uses_physical_ppem_buckets_for_ui_face() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let glyph_index = font
        .font()
        .expect("ui retained-host font")
        .lookup_glyph_index('P');
    let rasters = [1.0_f32, 1.25, 1.5, 2.0].map(|surface_scale_factor| {
        rasterize_cached_glyph(
            HostTextFontFace::Ui,
            glyph_index,
            13.0,
            surface_scale_factor,
            0.0,
        )
    });

    assert_eq!(
        rasters
            .iter()
            .map(|raster| raster.raster_px_size)
            .collect::<Vec<_>>(),
        vec![13, 16, 20, 26]
    );
    for raster in &rasters {
        assert_eq!(raster.source, CachedGlyphRasterSource::Swash);
        assert!(
            matches!(
                raster.format,
                CachedGlyphRasterFormat::AlphaMask | CachedGlyphRasterFormat::SubpixelMask
            ),
            "swash may return either alpha or subpixel coverage depending on the font backend"
        );
        assert_eq!(raster.sample_scale, 1.0);
        assert!(raster.metrics.width > 0);
        assert!(raster.metrics.height > 0);
        assert!(raster.bitmap.iter().any(|coverage| *coverage > 0));
    }
    for pair in rasters.windows(2) {
        assert!(!Arc::ptr_eq(&pair[0].bitmap, &pair[1].bitmap));
    }
    assert!(rasters[0].metrics.height < rasters[3].metrics.height);
}

#[test]
fn retained_text_raster_cache_reuses_equivalent_physical_ppem() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let glyph_index = font
        .font()
        .expect("ui retained-host font")
        .lookup_glyph_index('P');
    let scaled = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 1.25, 0.0);
    let physical = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 16.0, 1.0, 0.0);

    assert_eq!(scaled.raster_px_size, 16);
    assert_eq!(physical.raster_px_size, 16);
    assert!(Arc::ptr_eq(&scaled.bitmap, &physical.bitmap));
}

#[test]
fn retained_text_raster_cache_publication_tracks_residency_and_duplicate_misses() {
    fn key(glyph_index: u16) -> GlyphRasterKey {
        GlyphRasterKey {
            font_source: GlyphRasterFontSource::Host(HostTextFontFace::Ui),
            font_cache_key: 1,
            glyph_index,
            raster_px_size: 13,
            subpixel_offset_bits: 0.0_f32.to_bits(),
            text_smoothing: HostTextSmoothing::Grayscale,
        }
    }

    fn raster(bitmap: &[u8]) -> CachedGlyphRaster {
        CachedGlyphRaster {
            metrics: CachedGlyphMetrics::default(),
            bitmap: Arc::from(bitmap),
            source: CachedGlyphRasterSource::Swash,
            format: CachedGlyphRasterFormat::AlphaMask,
            raster_px_size: 13,
            sample_scale: 1.0,
            sample_offset_x: 0.0,
        }
    }

    let mut cache = GlyphRasterCache::default();
    let first = cache.publish_profiled(key(1), raster(&[1, 2, 3]));
    assert!(!first.duplicate);
    assert_eq!(first.entry_count, 1);
    assert_eq!(first.resident_bitmap_bytes, 3);
    assert_eq!(first.peak_entry_count, 1);
    assert_eq!(first.peak_resident_bitmap_bytes, 3);

    let replacement = cache.publish_profiled(key(1), raster(&[]));
    assert!(replacement.duplicate);
    assert_eq!(replacement.entry_count, 1);
    assert_eq!(replacement.resident_bitmap_bytes, 0);
    assert_eq!(replacement.peak_resident_bitmap_bytes, 3);

    let empty_duplicate = cache.publish_profiled(key(1), raster(&[]));
    assert!(empty_duplicate.duplicate);

    let second = cache.publish_profiled(key(2), raster(&[4, 5]));
    assert!(!second.duplicate);
    assert_eq!(second.entry_count, 2);
    assert_eq!(second.resident_bitmap_bytes, 2);
    assert_eq!(second.peak_entry_count, 2);
    assert_eq!(second.peak_resident_bitmap_bytes, 3);
}

#[test]
fn swash_subpixel_mask_preserves_rgb_coverage_for_retained_host() {
    let (format, bitmap) = swash_bitmap(
        Content::SubpixelMask,
        Source::Outline,
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
        Source::Outline,
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
fn swash_color_bitmap_preserves_its_rgba_pixels() {
    let pixels = vec![12, 34, 56, 78, 90, 120, 150, 255];
    let (format, bitmap) = swash_bitmap(
        Content::Color,
        Source::ColorBitmap(StrikeWith::BestFit),
        pixels.clone(),
        2,
        1,
        HostTextSmoothing::Grayscale,
    )
    .expect("color bitmap should stay renderable");

    assert_eq!(format, CachedGlyphRasterFormat::ColorRgba);
    assert_eq!(bitmap, pixels);
}

#[test]
fn swash_color_outline_is_unpremultiplied_for_straight_alpha_blending() {
    let (format, bitmap) = swash_bitmap(
        Content::Color,
        Source::ColorOutline(0),
        vec![64, 32, 16, 128, 7, 9, 11, 0],
        2,
        1,
        HostTextSmoothing::Grayscale,
    )
    .expect("color outline should stay renderable");

    assert_eq!(format, CachedGlyphRasterFormat::ColorRgba);
    assert_eq!(bitmap, vec![128, 64, 32, 128, 0, 0, 0, 0]);
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
fn retained_text_raster_disables_swash_hinting_for_compact_physical_labels() {
    assert!(!swash_hinting_for_physical_size(8.5));
    assert!(!swash_hinting_for_physical_size(10.0));
    assert!(
        !swash_hinting_for_physical_size(13.0),
        "13px editor tab/file labels should keep unhinted swash bearings so glyphs do not snap left/right independently"
    );
    assert!(swash_hinting_for_physical_size(13.01));
}

#[test]
fn retained_text_raster_cache_separates_subpixel_bins_for_same_glyph() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let glyph_index = font
        .font()
        .expect("ui retained-host font")
        .lookup_glyph_index('e');

    let left = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 1.0, 0.0);
    let right = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, 13.0, 1.0, 2.0 / 3.0);

    assert!(!Arc::ptr_eq(&left.bitmap, &right.bitmap));
    assert_eq!(left.source, CachedGlyphRasterSource::Swash);
    assert_eq!(right.source, CachedGlyphRasterSource::Swash);
}

#[test]
fn retained_text_raster_keeps_swash_for_tiny_editor_label_glyphs() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let font = font.font().expect("ui retained-host font");
    for (text, px) in [
        ("editor base.zui", 10.0_f32),
        ("folder-op...line.svg", 8.5_f32),
    ] {
        for character in text.chars().filter(|character| !character.is_whitespace()) {
            let glyph_index = font.lookup_glyph_index(character);
            let raster = rasterize_cached_glyph(HostTextFontFace::Ui, glyph_index, px, 1.0, 0.875);

            assert_eq!(
                raster.source,
                CachedGlyphRasterSource::Swash,
                "tiny editor label glyph '{character}' in {text:?} at {px}px must keep the same swash rasterizer instead of mixing fallback bearings"
            );
        }
    }
}

#[test]
fn retained_text_raster_normalizes_invalid_subpixel_offsets() {
    assert_eq!(normalized_subpixel_offset(f32::NAN), 0.0);
    assert_eq!(normalized_subpixel_offset(-1.0), 0.0);
    assert_eq!(normalized_subpixel_offset(2.0), 0.999);
}

#[test]
fn swash_metrics_x_offset_is_relative_to_pen_origin() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let glyph_index = font
        .font()
        .expect("ui retained-host font")
        .lookup_glyph_index('j');
    let metrics = swash_metrics(&font, glyph_index, 13.0, 7, 11, -2, 9);

    assert_eq!(metrics.x_offset, -2);
}

#[test]
fn fontdue_fallback_metrics_x_offset_is_relative_to_pen_origin() {
    let font = host_font_snapshot_for_face(HostTextFontFace::Ui);
    let glyph_index = font
        .font()
        .expect("ui retained-host font")
        .lookup_glyph_index('j');
    let raster_px_size = 39;
    let (raster_metrics, _) = font
        .font()
        .expect("ui retained-host font")
        .rasterize_indexed(glyph_index, raster_px_size as f32);
    let raster_left_px = raster_metrics.xmin as f32;
    let expected_x_offset = raster_left_px.floor() as i32;

    let raster = rasterize_fontdue_glyph(&font, glyph_index, raster_px_size, 0.5);

    assert_eq!(raster.metrics.x_offset, expected_x_offset);
    assert_eq!(
        raster.sample_offset_x,
        fontdue_fallback_sample_offset_x(0.5, raster_left_px, expected_x_offset),
        "fontdue fallback must preserve both pen-origin phase and fractional bitmap-left bearing"
    );
}

#[test]
fn fontdue_fallback_sample_offset_keeps_bitmap_left_fraction() {
    assert_eq!(fontdue_fallback_sample_offset_x(0.5, -0.375, -1), 1.125);
    assert_eq!(fontdue_fallback_sample_offset_x(0.25, 1.75, 1), 1.0);
}

#[test]
fn visible_low_coverage_swash_masks_remain_usable() {
    assert!(bitmap_has_visible_ink(
        CachedGlyphRasterFormat::AlphaMask,
        &[1]
    ));
    assert!(bitmap_has_visible_ink(
        CachedGlyphRasterFormat::SubpixelMask,
        &[0, 64, 0]
    ));
    assert!(!bitmap_has_visible_ink(
        CachedGlyphRasterFormat::AlphaMask,
        &[]
    ));
    assert!(!bitmap_has_visible_ink(
        CachedGlyphRasterFormat::AlphaMask,
        &[0, 0, 0]
    ));
    assert!(bitmap_has_visible_ink(
        CachedGlyphRasterFormat::ColorRgba,
        &[12, 34, 56, 1]
    ));
    assert!(!bitmap_has_visible_ink(
        CachedGlyphRasterFormat::ColorRgba,
        &[12, 34, 56, 0]
    ));
}

#[test]
fn fontdue_fallback_returns_empty_raster_when_font_is_unavailable() {
    let raster = empty_fontdue_raster(4, 0.5);

    assert_eq!(raster.metrics, CachedGlyphMetrics::default());
    assert!(raster.bitmap.is_empty());
    assert_eq!(raster.source, CachedGlyphRasterSource::FontdueFallback);
    assert_eq!(raster.raster_px_size, 4);
    assert_eq!(raster.sample_scale, 1.0);
    assert_eq!(raster.sample_offset_x, 0.5);
}
