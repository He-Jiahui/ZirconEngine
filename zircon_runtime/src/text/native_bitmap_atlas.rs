mod frame;
mod handoff;
mod raster_key;
mod report;
mod retry_frame;
mod source_cache;
mod source_image;
mod storage;
mod text_area;

#[cfg(test)]
use frame::NativeBitmapAtlasSourceImage;
pub(crate) use frame::{bitmap_atlas_page_size, native_bitmap_atlas_frame, NativeBitmapAtlasFrame};
pub(crate) use handoff::{
    native_bitmap_atlas_first_frame_degradation_for_report,
    native_bitmap_atlas_glyphon_fallback_reason_for_report, native_bitmap_atlas_handoff_for_report,
    NativeBitmapAtlasFirstFrameDegradation, NativeBitmapAtlasGlyphonFallbackReason,
    NativeBitmapAtlasHandoff,
};
pub(crate) use report::{native_bitmap_atlas_idle_prepare_report, NativeBitmapAtlasPrepareReport};
pub(crate) use source_cache::{
    NativeBitmapAtlasSourceCache, NativeBitmapAtlasSourceCacheFrameReport,
    NativeBitmapAtlasWorkerRequestStatus, NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME,
};
pub(crate) use storage::NativeBitmapAtlasStorageSubmission;
pub(crate) use text_area::NativeBitmapAtlasTextArea;

#[cfg(test)]
mod tests;
