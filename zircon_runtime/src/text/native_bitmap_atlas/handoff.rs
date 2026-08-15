use super::report::NativeBitmapAtlasPrepareReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasHandoff {
    SingleStorageReplacement,
    MixedStorageReplacement,
    NoVisibleGlyphs,
    TransparentPlaceholder,
    GlyphonFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasGlyphonFallbackReason {
    MissingRasterImage,
    RetryByteBudgetRejected,
    RetryQueueCapacityExceeded,
    NoVisibleRasterGlyphs,
    UnsupportedGlyphFormat,
    IncompleteSourceCoverage,
    MissingBackgroundCompositeInput,
    AtlasAllocationFailure,
    MixedStorageSplitNotReady,
    IncompleteStorageSubmission,
    MissingGpuDrawPlan,
    MissingSingleStorageFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasFirstFrameDegradation {
    ApproximateBucketReplacement,
    GlyphonFallback,
    TransparentPlaceholder,
}

pub(crate) fn native_bitmap_atlas_handoff_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> NativeBitmapAtlasHandoff {
    if report.replaces_glyphon {
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    } else if report.mixed_storage_replacement_ready {
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    } else if report.visible_raster_glyph_count == 0
        && report.visible_missing_raster_image_count == 0
        && report.unsupported_glyph_count == 0
        && !report.submission.has_placeholder_work()
    {
        NativeBitmapAtlasHandoff::NoVisibleGlyphs
    } else if report.retry_submission.has_byte_budget_rejections() {
        NativeBitmapAtlasHandoff::GlyphonFallback
    } else if report.retry_state.has_queue_overflow() {
        NativeBitmapAtlasHandoff::GlyphonFallback
    } else if report.submission.has_placeholder_work() {
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    } else {
        NativeBitmapAtlasHandoff::GlyphonFallback
    }
}

pub(crate) fn native_bitmap_atlas_glyphon_fallback_reason_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> Option<NativeBitmapAtlasGlyphonFallbackReason> {
    if !matches!(
        native_bitmap_atlas_handoff_for_report(report),
        NativeBitmapAtlasHandoff::GlyphonFallback
    ) {
        return None;
    }

    if report.visible_missing_raster_image_count > 0 {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::MissingRasterImage);
    }
    if report.retry_submission.has_byte_budget_rejections() {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::RetryByteBudgetRejected);
    }
    if report.retry_state.has_queue_overflow() {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::RetryQueueCapacityExceeded);
    }
    if report.visible_raster_glyph_count == 0 {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::NoVisibleRasterGlyphs);
    }
    if report.unsupported_glyph_count > 0 {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::UnsupportedGlyphFormat);
    }
    if report.source_image_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::IncompleteSourceCoverage);
    }
    if report.requires_background_composite && !report.background_composite_replacement_ready {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::MissingBackgroundCompositeInput);
    }
    if report.submission.allocation_failure_count > 0 {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::AtlasAllocationFailure);
    }
    if report.mixed_atlas_storage_format && !report.mixed_storage_replacement_ready {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::MixedStorageSplitNotReady);
    }
    if report.storage_submission_visible_glyph_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::IncompleteStorageSubmission);
    }
    if report.submission.visible_glyph_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::MissingGpuDrawPlan);
    }
    if report.atlas_storage_format.is_none() {
        return Some(NativeBitmapAtlasGlyphonFallbackReason::MissingSingleStorageFormat);
    }

    Some(NativeBitmapAtlasGlyphonFallbackReason::MissingGpuDrawPlan)
}

pub(crate) fn native_bitmap_atlas_first_frame_degradation_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> Option<NativeBitmapAtlasFirstFrameDegradation> {
    match native_bitmap_atlas_handoff_for_report(report) {
        NativeBitmapAtlasHandoff::SingleStorageReplacement
        | NativeBitmapAtlasHandoff::MixedStorageReplacement => {
            return (report.approximate_raster_image_count > 0)
                .then_some(NativeBitmapAtlasFirstFrameDegradation::ApproximateBucketReplacement);
        }
        NativeBitmapAtlasHandoff::NoVisibleGlyphs => return None,
        NativeBitmapAtlasHandoff::TransparentPlaceholder => {
            return Some(NativeBitmapAtlasFirstFrameDegradation::TransparentPlaceholder);
        }
        NativeBitmapAtlasHandoff::GlyphonFallback => {}
    }

    if report.visible_missing_raster_image_count > 0
        || report.source_image_count < report.visible_raster_glyph_count
    {
        return Some(NativeBitmapAtlasFirstFrameDegradation::GlyphonFallback);
    }

    None
}
