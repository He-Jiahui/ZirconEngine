mod frame_driver;
mod frame_state;
mod placeholder;
mod plan;
mod report;
mod retry;

pub(crate) use frame_driver::{
    GlyphAtlasBitmapRetryFrameDriverConfig, GlyphAtlasBitmapRetryFrameDriverOutput,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_config,
};
pub(crate) use frame_state::{
    GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport,
};
pub(crate) use placeholder::{
    GlyphAtlasBitmapPlaceholderDraw, GlyphAtlasBitmapPlaceholderDrawPlan,
    glyph_atlas_bitmap_placeholder_draw_plan,
};
pub(crate) use plan::{
    GlyphAtlasBitmapRenderSubmissionPlan, glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
    glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding,
    glyph_atlas_bitmap_render_submission_plan_with_padding,
};
pub(crate) use report::{
    GlyphAtlasBitmapRenderSubmissionReport, glyph_atlas_bitmap_render_submission_report,
};
pub(crate) use retry::{
    GlyphAtlasBitmapRetryFrameSubmissionPlan, GlyphAtlasBitmapRetryFrameSubmissionReport,
    glyph_atlas_bitmap_retry_frame_submission_plan,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding,
    glyph_atlas_bitmap_retry_frame_submission_report,
};

#[cfg(test)]
mod tests;
