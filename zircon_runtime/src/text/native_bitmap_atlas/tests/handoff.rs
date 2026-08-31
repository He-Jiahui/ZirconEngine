use super::*;
use crate::text::atlas::GlyphAtlasBitmapRetryFrameSubmissionReport;

#[test]
fn native_bitmap_atlas_handoff_uses_single_storage_native_submission() {
    let report = NativeBitmapAtlasPrepareReport {
        native_submission_ready: true,
        mixed_storage_replacement_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
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
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_handoff_stays_idle_when_no_raster_glyph_is_visible() {
    let report = NativeBitmapAtlasPrepareReport::default();

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_handoff_does_not_report_offscreen_approximation_as_degradation() {
    let report = NativeBitmapAtlasPrepareReport {
        approximate_raster_image_count: 1,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_handoff_stays_idle_for_retry_pressure_without_visible_work() {
    let report = NativeBitmapAtlasPrepareReport {
        retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport {
            rejected_new_source_count: 1,
            rejected_new_source_byte_count: 1024 * 1024 + 1,
            ..GlyphAtlasBitmapRetryFrameSubmissionReport::default()
        },
        retry_state: GlyphAtlasBitmapRetryFrameStateReport {
            queue_overflow_blocked_glyph_count: 1,
            queue_overflow_blocked_source_byte_count: 64,
            ..GlyphAtlasBitmapRetryFrameStateReport::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_handoff_preserves_placeholder_work_without_visible_raster_glyphs() {
    let report = NativeBitmapAtlasPrepareReport {
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
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder)
    );
}

#[test]
fn native_bitmap_atlas_degradation_reports_missing_raster_image_first() {
    let report = NativeBitmapAtlasPrepareReport {
        missing_raster_image_count: 1,
        visible_missing_raster_image_count: 1,
        visible_raster_glyph_count: 0,
        source_image_count: 0,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::Degraded
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::MissingRasterImage)
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable)
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
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder)
    );
}

#[test]
fn native_bitmap_atlas_handoff_degrades_for_missing_background_composite() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 1,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        requires_background_composite: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::Degraded
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::MissingBackgroundCompositeInput)
    );
}

#[test]
fn native_bitmap_atlas_handoff_uses_subpixel_replacement_when_background_ready() {
    let report = NativeBitmapAtlasPrepareReport {
        requires_background_composite: true,
        background_composite_replacement_ready: true,
        native_submission_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        None
    );
}

#[test]
fn native_bitmap_atlas_degradation_reports_incomplete_source_coverage() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 2,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::IncompleteSourceCoverage)
    );
}

#[test]
fn native_bitmap_atlas_degradation_reports_terminal_retry_byte_budget_rejection() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 1,
        source_image_count: 0,
        retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport {
            rejected_new_source_count: 1,
            rejected_new_source_byte_count: 1024 * 1024 + 1,
            ..GlyphAtlasBitmapRetryFrameSubmissionReport::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::Degraded
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::RetryByteBudgetRejected)
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable)
    );
}

#[test]
fn native_bitmap_atlas_byte_budget_rejection_overrides_unrelated_placeholder_work() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 2,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        submission: GlyphAtlasBitmapRenderSubmissionReport {
            visible_placeholder_count: 1,
            ..GlyphAtlasBitmapRenderSubmissionReport::default()
        },
        retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport {
            rejected_new_source_count: 1,
            rejected_new_source_byte_count: 1024 * 1024 + 1,
            ..GlyphAtlasBitmapRetryFrameSubmissionReport::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::Degraded
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::RetryByteBudgetRejected)
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable)
    );
}

#[test]
fn native_bitmap_atlas_retry_queue_overflow_overrides_placeholder_work() {
    let report = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 2,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        submission: GlyphAtlasBitmapRenderSubmissionReport {
            visible_placeholder_count: 1,
            ..GlyphAtlasBitmapRenderSubmissionReport::default()
        },
        retry_state: GlyphAtlasBitmapRetryFrameStateReport {
            queue_overflow_blocked_glyph_count: 1,
            queue_overflow_blocked_source_byte_count: 64,
            ..GlyphAtlasBitmapRetryFrameStateReport::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::Degraded
    );
    assert_eq!(
        native_bitmap_atlas_degradation_reason_for_report(&report),
        Some(NativeBitmapAtlasDegradationReason::RetryQueueCapacityExceeded)
    );
    assert_eq!(
        native_bitmap_atlas_first_frame_degradation_for_report(&report),
        Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable)
    );
}
