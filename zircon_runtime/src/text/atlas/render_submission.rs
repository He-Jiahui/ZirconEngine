mod frame_driver;
mod frame_state;
mod placeholder;
mod plan;
mod report;
mod retry;

pub(crate) use frame_driver::{
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_config,
    GlyphAtlasBitmapRetryFrameDriverConfig, GlyphAtlasBitmapRetryFrameDriverOutput,
};
pub(crate) use frame_state::{
    GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport,
};
pub(crate) use placeholder::{
    glyph_atlas_bitmap_placeholder_draw_plan, GlyphAtlasBitmapPlaceholderDraw,
    GlyphAtlasBitmapPlaceholderDrawPlan,
};
pub(crate) use plan::{
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
    glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding,
    glyph_atlas_bitmap_render_submission_plan_with_padding, GlyphAtlasBitmapRenderSubmissionPlan,
};
pub(crate) use report::{
    glyph_atlas_bitmap_render_submission_report, GlyphAtlasBitmapRenderSubmissionReport,
};
pub(crate) use retry::{
    glyph_atlas_bitmap_retry_frame_submission_plan,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding,
    glyph_atlas_bitmap_retry_frame_submission_report, GlyphAtlasBitmapRetryFrameSubmissionPlan,
    GlyphAtlasBitmapRetryFrameSubmissionReport,
};

#[cfg(test)]
mod tests;
