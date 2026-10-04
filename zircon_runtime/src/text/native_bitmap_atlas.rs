mod frame;
mod glyph_run;
mod handoff;
mod report;
mod retry_frame;
mod source_cache;
mod source_image;
mod storage;

#[cfg(test)]
use frame::NativeBitmapAtlasSourceImage;
pub(crate) use frame::{bitmap_atlas_page_size, native_bitmap_atlas_frame, NativeBitmapAtlasFrame};
pub(crate) use glyph_run::{NativeBitmapAtlasGlyph, NativeBitmapAtlasGlyphRun};
pub(crate) use handoff::{
    native_bitmap_atlas_degradation_reason_for_report,
    native_bitmap_atlas_first_frame_degradation_for_report, native_bitmap_atlas_handoff_for_report,
    NativeBitmapAtlasDegradationReason, NativeBitmapAtlasFirstFrameDegradation,
    NativeBitmapAtlasHandoff,
};
pub(crate) use report::{native_bitmap_atlas_idle_prepare_report, NativeBitmapAtlasPrepareReport};
pub(crate) use source_cache::{
    NativeBitmapAtlasReadinessChangeReceipt, NativeBitmapAtlasReadinessGeneration,
    NativeBitmapAtlasSourceCache, NativeBitmapAtlasSourceCacheFrameReport,
    NativeBitmapAtlasWorkerRequestStatus, NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME,
};

#[cfg(test)]
#[path = "native_bitmap_atlas/tests/cases.rs"]
mod tests;
