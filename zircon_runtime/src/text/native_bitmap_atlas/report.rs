use crate::text::atlas::{
    GlyphAtlasBitmapRenderSubmissionReport, GlyphAtlasBitmapRetryFrameState,
    GlyphAtlasBitmapRetryFrameStateReport, GlyphAtlasBitmapRetryFrameSubmissionReport,
    GlyphAtlasStorageFormat,
};

use super::handoff::{
    NativeBitmapAtlasFirstFrameDegradation, NativeBitmapAtlasGlyphonFallbackReason,
};
use super::source_cache::{NativeBitmapAtlasSourceCache, NativeBitmapAtlasSourceCacheFrameReport};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeBitmapAtlasPrepareReport {
    pub(crate) frame_index: u64,
    pub(crate) visible_raster_glyph_count: usize,
    pub(crate) source_image_count: usize,
    pub(crate) missing_raster_image_count: usize,
    /// Cache misses whose layout bounds intersect the current text area bounds.
    ///
    /// Offscreen misses remain cache diagnostics, but must not force glyphon for
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
    pub(crate) glyphon_fallback_reason: Option<NativeBitmapAtlasGlyphonFallbackReason>,
    pub(crate) first_frame_degradation: Option<NativeBitmapAtlasFirstFrameDegradation>,
    pub(crate) replaces_glyphon: bool,
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
