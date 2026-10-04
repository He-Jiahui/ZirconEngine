//! 把文字来源、图集分配、GPU 计划和异步缓存结果汇合成一次原生位图帧报告。
//! 准备报告是渲染交接决策输入；空闲报告只更新缓存/队列诊断，不制造虚假的可见绘制。

use crate::text::atlas::{
    GlyphAtlasBitmapRenderSubmissionReport, GlyphAtlasBitmapRetryFrameState,
    GlyphAtlasBitmapRetryFrameStateReport, GlyphAtlasBitmapRetryFrameSubmissionReport,
    GlyphAtlasStorageFormat,
};

use super::handoff::{NativeBitmapAtlasDegradationReason, NativeBitmapAtlasFirstFrameDegradation};
use super::source_cache::{NativeBitmapAtlasSourceCache, NativeBitmapAtlasSourceCacheFrameReport};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 同一帧各阶段共享的交接证据；来源数量、可见字形、提交可见数和背景合成就绪性必须一起判断。
/// `native_submission_ready` 由帧对象按完整条件产生，不能仅凭任一计数为正就宣称可替换文字。
pub(crate) struct NativeBitmapAtlasPrepareReport {
    pub(crate) frame_index: u64,
    pub(crate) visible_raster_glyph_count: usize,
    pub(crate) source_image_count: usize,
    pub(crate) missing_raster_image_count: usize,
    /// Cache misses whose layout bounds intersect the current text area bounds.
    ///
    /// Offscreen misses remain cache diagnostics, but must not force native degradation for
    /// an otherwise empty native-atlas frame.
    pub(crate) visible_missing_raster_image_count: usize,
    pub(crate) approximate_raster_image_count: usize,
    pub(crate) unsupported_glyph_count: usize,
    pub(crate) clipped_glyph_count: usize,
    pub(crate) atlas_storage_format: Option<GlyphAtlasStorageFormat>,
    pub(crate) mixed_atlas_storage_format: bool,
    pub(crate) storage_submission_count: usize,
    pub(crate) storage_submission_visible_glyph_count: usize,
    pub(crate) mixed_storage_replacement_ready: bool,
    pub(crate) requires_background_composite: bool,
    pub(crate) background_composite_replacement_ready: bool,
    pub(crate) background_composite_glyph_count: usize,
    pub(crate) missing_background_composite_glyph_count: usize,
    pub(crate) source_cache: NativeBitmapAtlasSourceCacheFrameReport,
    pub(crate) retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    pub(crate) retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    pub(crate) discarded_stale_retry_glyph_count: usize,
    pub(crate) native_degradation_reason: Option<NativeBitmapAtlasDegradationReason>,
    pub(crate) first_frame_degradation: Option<NativeBitmapAtlasFirstFrameDegradation>,
    pub(crate) native_submission_ready: bool,
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionReport,
}

pub(crate) fn native_bitmap_atlas_idle_prepare_report(
    source_cache: &mut NativeBitmapAtlasSourceCache,
    retry_state: &mut GlyphAtlasBitmapRetryFrameState,
) -> NativeBitmapAtlasPrepareReport {
    NativeBitmapAtlasPrepareReport {
        source_cache: source_cache.idle_frame_report(),
        retry_state: retry_state.take_report(),
        ..NativeBitmapAtlasPrepareReport::default()
    }
}
