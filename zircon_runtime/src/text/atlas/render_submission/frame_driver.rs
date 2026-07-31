use crate::core::math::UVec2;

use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GLYPH_BITMAP_ATLAS_PADDING_PX, GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapSource,
    GlyphAtlasSet,
};
use super::frame_state::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport};
use super::retry::{
    GlyphAtlasBitmapRetryFrameSubmissionPlan, GlyphAtlasBitmapRetryFrameSubmissionReport,
};

/// Immutable knobs for one bitmap-atlas retry frame handoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameDriverConfig {
    pub(crate) page_size: UVec2,
    pub(crate) max_pages_per_format: usize,
    pub(crate) padding_px: u32,
    pub(crate) backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    pub(crate) viewport_size: UVec2,
    pub(crate) clip_rect: GlyphAtlasScreenRect,
}

/// Output of one retry-frame handoff after the cross-frame queue was committed.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameDriverOutput {
    pub(crate) submission_plan: GlyphAtlasBitmapRetryFrameSubmissionPlan,
    pub(crate) state_report: GlyphAtlasBitmapRetryFrameStateReport,
}

impl GlyphAtlasBitmapRetryFrameDriverConfig {
    pub(crate) fn with_defaults(
        page_size: UVec2,
        max_pages_per_format: usize,
        viewport_size: UVec2,
        clip_rect: GlyphAtlasScreenRect,
    ) -> Self {
        Self {
            page_size,
            max_pages_per_format,
            padding_px: GLYPH_BITMAP_ATLAS_PADDING_PX,
            backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy::unlimited(),
            viewport_size,
            clip_rect,
        }
    }
}

impl GlyphAtlasBitmapRetryFrameDriverOutput {
    pub(crate) fn retry_submission_report(&self) -> GlyphAtlasBitmapRetryFrameSubmissionReport {
        self.submission_plan.retry_submission_report()
    }
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_driver_submit_with_config<S>(
    state: &mut GlyphAtlasBitmapRetryFrameState,
    frame_sources: S,
    frame_index: u64,
    config: GlyphAtlasBitmapRetryFrameDriverConfig,
) -> GlyphAtlasBitmapRetryFrameDriverOutput
where
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config(
        state,
        GlyphAtlasSet::default(),
        frame_sources,
        frame_index,
        config,
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config<S>(
    state: &mut GlyphAtlasBitmapRetryFrameState,
    atlas: GlyphAtlasSet,
    frame_sources: S,
    frame_index: u64,
    config: GlyphAtlasBitmapRetryFrameDriverConfig,
) -> GlyphAtlasBitmapRetryFrameDriverOutput
where
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let submission_plan = state.submission_plan_with_atlas_backpressure_and_padding(
        atlas,
        frame_sources,
        config.page_size,
        frame_index,
        config.max_pages_per_format,
        config.padding_px,
        config.backpressure_policy,
        config.viewport_size,
        config.clip_rect,
    );
    let state_report =
        state.apply_submission_plan_with_backpressure(&submission_plan, config.backpressure_policy);

    GlyphAtlasBitmapRetryFrameDriverOutput {
        submission_plan,
        state_report,
    }
}
