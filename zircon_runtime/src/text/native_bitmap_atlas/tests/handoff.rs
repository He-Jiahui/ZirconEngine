use super::*;

#[test]
fn native_bitmap_atlas_handoff_uses_single_storage_replacement() {
    let report = NativeBitmapAtlasPrepareReport {
        replaces_glyphon: true,
        mixed_storage_replacement_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_handoff_routes_mixed_storage_to_renderer_submissions() {
    let report = NativeBitmapAtlasPrepareReport {
        mixed_storage_replacement_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_glyphon_fallback_reports_missing_raster_image_first() {
    let report = NativeBitmapAtlasPrepareReport {
        missing_raster_image_count: 1,
        visible_raster_glyph_count: 0,
        source_image_count: 0,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::GlyphonFallback
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        Some(NativeBitmapAtlasGlyphonFallbackReason::MissingRasterImage)
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::GlyphonFallback)
    );
}

#[test]
fn native_bitmap_atlas_first_frame_degradation_reports_placeholder_work() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 1,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 0,
        submission: GlyphAtlasBitmapRenderSubmissionReport {
            visible_placeholder_count: 1,
            ..GlyphAtlasBitmapRenderSubmissionReport::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        None
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder)
    );
}

#[test]
fn native_bitmap_atlas_handoff_keeps_glyphon_for_background_composite() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 1,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        requires_background_composite: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::GlyphonFallback
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        Some(NativeBitmapAtlasGlyphonFallbackReason::MissingBackgroundCompositeInput)
    );
}

#[test]
fn native_bitmap_atlas_handoff_uses_subpixel_replacement_when_background_ready() {
    let report = NativeBitmapAtlasPrepareReport {
        requires_background_composite: true,
        background_composite_replacement_ready: true,
        replaces_glyphon: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_glyphon_fallback_reports_incomplete_source_coverage() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 2,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_glyphon_fallback_reason_for_report(&report),
        Some(NativeBitmapAtlasGlyphonFallbackReason::IncompleteSourceCoverage)
    );
}
