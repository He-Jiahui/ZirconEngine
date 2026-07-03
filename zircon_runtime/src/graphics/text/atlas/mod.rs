//! Shared glyph atlas contracts for bitmap, SDF, MSDF, and color text pages.

mod bitmap_run;
mod dirty;
mod page;
mod page_residency;
mod raster_key;
pub(crate) mod render_batch;
pub(crate) mod render_contract;
pub(crate) mod render_gpu_plan;
pub(crate) mod render_plan;
mod render_submission;
mod shelf_allocator;
mod upload;

pub(crate) use bitmap_run::{
    glyph_atlas_bitmap_run_plan, glyph_atlas_bitmap_run_plan_with_padding,
    GlyphAtlasBitmapAllocationFailure, GlyphAtlasBitmapAllocationFailureReason,
    GlyphAtlasBitmapGlyph, GlyphAtlasBitmapPlaceholderGlyph, GlyphAtlasBitmapPlaceholderMode,
    GlyphAtlasBitmapQueuedGlyph, GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource,
    GLYPH_BITMAP_ATLAS_PADDING_PX,
};
pub(crate) use dirty::GlyphAtlasDirtyPage;
pub(crate) use page::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasRect,
    GlyphAtlasSamplingSemantics, GlyphAtlasSet, GlyphAtlasStorageFormat,
};
pub(crate) use page_residency::{
    GlyphAtlasPageReservation, GlyphAtlasPageResidencyDecision,
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};
pub(crate) use raster_key::{
    GlyphHintingMode, GlyphRasterKey, GlyphRasterPlacement, GlyphRasterRequest, GlyphSmoothingMode,
    SyntheticGlyphStyle,
};
pub(crate) use render_submission::{
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_padding,
    glyph_atlas_bitmap_render_submission_report, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapRenderSubmissionReport,
};
pub(crate) use shelf_allocator::{GlyphAtlasAllocation, GlyphAtlasShelfAllocator};
pub(crate) use upload::{
    glyph_atlas_upload_command, GlyphAtlasUploadCommand, GlyphAtlasUploadMode,
};
