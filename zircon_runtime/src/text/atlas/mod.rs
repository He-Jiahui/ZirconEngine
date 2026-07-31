//! Shared glyph atlas contracts for bitmap, SDF, MSDF, and color text pages.

mod bitmap_run;
mod dirty;
mod page;
mod page_residency;
mod page_shadow;
mod raster_key;
pub(crate) mod render_batch;
pub(crate) mod render_contract;
pub(crate) mod render_gpu_plan;
pub(crate) mod render_plan;
mod render_submission;
mod shelf_allocator;
mod slot_cache;
mod upload;

pub(crate) use bitmap_run::{
    GLYPH_BITMAP_ATLAS_PADDING_PX, GlyphAtlasBitmapAllocationFailure,
    GlyphAtlasBitmapAllocationFailureReason, GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapGlyph,
    GlyphAtlasBitmapPageUploadStaging, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapQueuedGlyph, GlyphAtlasBitmapRequeueReason, GlyphAtlasBitmapRequeuedUpload,
    GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapRetryFrameInput,
    GlyphAtlasBitmapRetryFrameOutcome, GlyphAtlasBitmapRetryPlan,
    GlyphAtlasBitmapRetrySourceOrigin, GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSlotInvalidation,
    GlyphAtlasBitmapSource, GlyphAtlasBitmapStagedUpload, GlyphAtlasBitmapStagedUploadFailure,
    GlyphAtlasBitmapStagedUploadFailureReason, GlyphAtlasBitmapStagedUploadPlan,
    GlyphAtlasBitmapTextureUploadRequest, GlyphAtlasBitmapTextureUploadRequestPlan,
    GlyphAtlasBitmapUploadCopy, GlyphAtlasBitmapUploadSourceBytes,
    GlyphAtlasBitmapUploadStagingFailure, GlyphAtlasBitmapUploadStagingFailureReason,
    GlyphAtlasBitmapUploadStagingPlan, glyph_atlas_bitmap_page_shadow_commit,
    glyph_atlas_bitmap_prepared_upload_plan, glyph_atlas_bitmap_retry_frame_input,
    glyph_atlas_bitmap_retry_frame_input_with_backpressure, glyph_atlas_bitmap_retry_frame_outcome,
    glyph_atlas_bitmap_retry_plan, glyph_atlas_bitmap_retry_plan_with_backpressure,
    glyph_atlas_bitmap_run_plan, glyph_atlas_bitmap_run_plan_with_atlas,
    glyph_atlas_bitmap_run_plan_with_atlas_and_padding, glyph_atlas_bitmap_run_plan_with_padding,
    glyph_atlas_bitmap_staged_upload_plan, glyph_atlas_bitmap_texture_upload_request_plan,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity,
    glyph_atlas_bitmap_upload_staging_plan,
};
pub(crate) use dirty::GlyphAtlasDirtyPage;
pub(crate) use page::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasRect,
    GlyphAtlasSamplingSemantics, GlyphAtlasSet, GlyphAtlasStorageFormat,
};
pub(crate) use page_residency::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasPageReservation,
    GlyphAtlasPageResidencyDecision,
};
pub(crate) use page_shadow::{GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowPatch};
pub(crate) use raster_key::{
    GlyphHintingMode, GlyphRasterKey, GlyphRasterPlacement, GlyphRasterRequest, GlyphSmoothingMode,
    SyntheticGlyphStyle,
};
pub(crate) use render_submission::{
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapRenderSubmissionReport,
    GlyphAtlasBitmapRetryFrameDriverConfig, GlyphAtlasBitmapRetryFrameDriverOutput,
    GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport,
    GlyphAtlasBitmapRetryFrameSubmissionPlan, GlyphAtlasBitmapRetryFrameSubmissionReport,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
    glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding,
    glyph_atlas_bitmap_render_submission_plan_with_padding,
    glyph_atlas_bitmap_render_submission_report,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_config,
    glyph_atlas_bitmap_retry_frame_submission_plan,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding,
    glyph_atlas_bitmap_retry_frame_submission_report,
};
pub(crate) use shelf_allocator::{GlyphAtlasAllocation, GlyphAtlasShelfAllocator};
pub(crate) use upload::{
    GlyphAtlasUploadCommand, GlyphAtlasUploadMode, glyph_atlas_upload_command,
};
