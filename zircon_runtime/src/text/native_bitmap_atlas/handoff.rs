//! 依据一帧准备报告选择原生图集绘制、空帧、透明占位或退化。
//! 判定顺序保留可替换的完整帧计划；缺像素、预算拒绝及子像素背景不完整时禁止把部分成功当完整文字。

use super::report::NativeBitmapAtlasPrepareReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasHandoff {
    SingleStorageReplacement,
    MixedStorageReplacement,
    NoVisibleGlyphs,
    TransparentPlaceholder,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeBitmapAtlasDegradationReason {
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
    NativeRasterUnavailable,
    TransparentPlaceholder,
}

/// 由 UI 文字渲染准备路径在提交纹理前调用；报告须来自同一个完整帧计划。
/// 纯离屏缺失不迫使可见空帧退化，可见占位与预算溢出则按优先级决定后续绘制策略。
pub(crate) fn native_bitmap_atlas_handoff_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> NativeBitmapAtlasHandoff {
    if report.native_submission_ready {
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
        NativeBitmapAtlasHandoff::Degraded
    } else if report.retry_state.has_queue_overflow() {
        NativeBitmapAtlasHandoff::Degraded
    } else if report.submission.has_placeholder_work() {
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    } else {
        NativeBitmapAtlasHandoff::Degraded
    }
}

pub(crate) fn native_bitmap_atlas_degradation_reason_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> Option<NativeBitmapAtlasDegradationReason> {
    if !matches!(
        native_bitmap_atlas_handoff_for_report(report),
        NativeBitmapAtlasHandoff::Degraded
    ) {
        return None;
    }

    if report.visible_missing_raster_image_count > 0 {
        return Some(NativeBitmapAtlasDegradationReason::MissingRasterImage);
    }
    if report.retry_submission.has_byte_budget_rejections() {
        return Some(NativeBitmapAtlasDegradationReason::RetryByteBudgetRejected);
    }
    if report.retry_state.has_queue_overflow() {
        return Some(NativeBitmapAtlasDegradationReason::RetryQueueCapacityExceeded);
    }
    if report.visible_raster_glyph_count == 0 {
        return Some(NativeBitmapAtlasDegradationReason::NoVisibleRasterGlyphs);
    }
    if report.unsupported_glyph_count > 0 {
        return Some(NativeBitmapAtlasDegradationReason::UnsupportedGlyphFormat);
    }
    if report.source_image_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasDegradationReason::IncompleteSourceCoverage);
    }
    if report.requires_background_composite && !report.background_composite_replacement_ready {
        return Some(NativeBitmapAtlasDegradationReason::MissingBackgroundCompositeInput);
    }
    if report.submission.allocation_failure_count > 0 {
        return Some(NativeBitmapAtlasDegradationReason::AtlasAllocationFailure);
    }
    if report.mixed_atlas_storage_format && !report.mixed_storage_replacement_ready {
        return Some(NativeBitmapAtlasDegradationReason::MixedStorageSplitNotReady);
    }
    if report.storage_submission_visible_glyph_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasDegradationReason::IncompleteStorageSubmission);
    }
    if report.submission.visible_glyph_count != report.visible_raster_glyph_count {
        return Some(NativeBitmapAtlasDegradationReason::MissingGpuDrawPlan);
    }
    if report.atlas_storage_format.is_none() {
        return Some(NativeBitmapAtlasDegradationReason::MissingSingleStorageFormat);
    }

    Some(NativeBitmapAtlasDegradationReason::MissingGpuDrawPlan)
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
        NativeBitmapAtlasHandoff::Degraded => {}
    }

    if report.visible_missing_raster_image_count > 0
        || report.source_image_count < report.visible_raster_glyph_count
    {
        return Some(NativeBitmapAtlasFirstFrameDegradation::NativeRasterUnavailable);
    }

    None
}
