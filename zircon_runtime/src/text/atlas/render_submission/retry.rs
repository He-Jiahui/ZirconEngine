use crate::core::math::UVec2;

use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GLYPH_BITMAP_ATLAS_PADDING_PX, GlyphAtlasBitmapQueuedGlyph,
    GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapRetryFrameInput,
    GlyphAtlasBitmapRetryFrameOutcome, GlyphAtlasBitmapSource, GlyphAtlasSet,
    glyph_atlas_bitmap_retry_frame_input_with_backpressure_and_new_source_budget_predicate,
    glyph_atlas_bitmap_retry_frame_outcome,
};
use super::plan::{
    GlyphAtlasBitmapRenderSubmissionPlan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding,
};
use super::report::GlyphAtlasBitmapRenderSubmissionReport;

/// Renderer-facing retry handoff: input selection, submission work, and next retry state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameSubmissionPlan {
    pub(crate) frame_input: GlyphAtlasBitmapRetryFrameInput,
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    pub(crate) frame_outcome: GlyphAtlasBitmapRetryFrameOutcome,
}

/// Compact telemetry for retry-aware submission without exposing run-local source indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameSubmissionReport {
    pub(crate) input_source_count: usize,
    pub(crate) retried_source_count: usize,
    pub(crate) retried_source_byte_count: usize,
    pub(crate) new_source_count: usize,
    pub(crate) new_source_byte_count: usize,
    pub(crate) budgeted_new_source_count: usize,
    pub(crate) budgeted_new_source_byte_count: usize,
    pub(crate) deferred_retry_count: usize,
    pub(crate) deferred_retry_source_byte_count: usize,
    pub(crate) backpressured_retry_count: usize,
    pub(crate) backpressured_retry_source_byte_count: usize,
    pub(crate) rejected_retry_source_count: usize,
    pub(crate) rejected_retry_source_byte_count: usize,
    pub(crate) deferred_new_source_count: usize,
    pub(crate) deferred_new_source_byte_count: usize,
    pub(crate) backpressured_new_source_count: usize,
    pub(crate) backpressured_new_source_byte_count: usize,
    pub(crate) rejected_new_source_count: usize,
    pub(crate) rejected_new_source_byte_count: usize,
    pub(crate) submission_report: GlyphAtlasBitmapRenderSubmissionReport,
    pub(crate) completed_retried_source_count: usize,
    pub(crate) completed_new_source_count: usize,
    pub(crate) blocked_retried_source_count: usize,
    pub(crate) blocked_new_source_count: usize,
    pub(crate) next_blocked_glyph_count: usize,
    pub(crate) next_blocked_new_source_count: usize,
    pub(crate) unmapped_blocked_source_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
}

impl GlyphAtlasBitmapRetryFrameSubmissionPlan {
    pub(crate) fn retry_submission_report(&self) -> GlyphAtlasBitmapRetryFrameSubmissionReport {
        glyph_atlas_bitmap_retry_frame_submission_report(self)
    }
}

impl GlyphAtlasBitmapRetryFrameSubmissionReport {
    pub(crate) fn has_retry_input(self) -> bool {
        self.retried_source_count > 0
    }

    pub(crate) fn has_pending_retry_work(self) -> bool {
        self.next_blocked_glyph_count > 0
    }

    pub(crate) fn has_blocked_retry_work(self) -> bool {
        self.blocked_retried_source_count > 0 || self.blocked_new_source_count > 0
    }

    pub(crate) fn has_backpressured_retry_work(self) -> bool {
        self.backpressured_retry_count > 0
    }

    pub(crate) fn has_backpressured_new_work(self) -> bool {
        self.backpressured_new_source_count > 0
    }

    pub(crate) fn has_unmapped_blocked_sources(self) -> bool {
        self.unmapped_blocked_source_count > 0
    }

    pub(crate) fn has_byte_budget_rejections(self) -> bool {
        self.rejected_retry_source_count > 0 || self.rejected_new_source_count > 0
    }
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_plan<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
        blocked_glyphs,
        frame_sources,
        page_size,
        frame_index,
        max_pages_per_format,
        GLYPH_BITMAP_ATLAS_PADDING_PX,
        viewport_size,
        clip_rect,
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_plan_with_padding<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        blocked_glyphs,
        frame_sources,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
        GlyphAtlasBitmapRetryBackpressurePolicy::unlimited(),
        viewport_size,
        clip_rect,
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
        blocked_glyphs,
        frame_sources,
        page_size,
        frame_index,
        max_pages_per_format,
        GLYPH_BITMAP_ATLAS_PADDING_PX,
        backpressure_policy,
        viewport_size,
        clip_rect,
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
        GlyphAtlasSet::default(),
        blocked_glyphs,
        frame_sources,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
        backpressure_policy,
        viewport_size,
        clip_rect,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding<
    R,
    S,
>(
    atlas: GlyphAtlasSet,
    blocked_glyphs: R,
    frame_sources: S,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let mut atlas = atlas;
    let frame_input =
        glyph_atlas_bitmap_retry_frame_input_with_backpressure_and_new_source_budget_predicate(
            blocked_glyphs,
            frame_sources,
            frame_index,
            backpressure_policy,
            |source| {
                max_pages_per_format == 0
                    || source.raster_key.is_none_or(|key| {
                        atlas
                            .persistent_bitmap_slot(
                                key,
                                source.content_size,
                                page_size,
                                frame_index,
                            )
                            .is_none()
                    })
            },
        );
    let submission = glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding(
        atlas,
        frame_input.sources.iter().copied(),
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
        viewport_size,
        clip_rect,
    );
    let frame_outcome = glyph_atlas_bitmap_retry_frame_outcome(&frame_input, &submission.run);

    GlyphAtlasBitmapRetryFrameSubmissionPlan {
        frame_input,
        submission,
        frame_outcome,
    }
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_submission_report(
    plan: &GlyphAtlasBitmapRetryFrameSubmissionPlan,
) -> GlyphAtlasBitmapRetryFrameSubmissionReport {
    GlyphAtlasBitmapRetryFrameSubmissionReport {
        input_source_count: plan.frame_input.sources.len(),
        retried_source_count: plan.frame_input.retried_source_count,
        retried_source_byte_count: plan.frame_input.retried_source_byte_count,
        new_source_count: plan.frame_input.new_source_count,
        new_source_byte_count: plan.frame_input.new_source_byte_count,
        budgeted_new_source_count: plan.frame_input.budgeted_new_source_count,
        budgeted_new_source_byte_count: plan.frame_input.budgeted_new_source_byte_count,
        deferred_retry_count: plan.frame_input.deferred_retry_count,
        deferred_retry_source_byte_count: plan.frame_input.deferred_retry_source_byte_count,
        backpressured_retry_count: plan.frame_input.backpressured_retry_count,
        backpressured_retry_source_byte_count: plan
            .frame_input
            .backpressured_retry_source_byte_count,
        rejected_retry_source_count: plan.frame_input.rejected_retry_source_count,
        rejected_retry_source_byte_count: plan.frame_input.rejected_retry_source_byte_count,
        deferred_new_source_count: plan.frame_input.deferred_new_source_count,
        deferred_new_source_byte_count: plan.frame_input.deferred_new_source_byte_count,
        backpressured_new_source_count: plan.frame_input.backpressured_new_source_count,
        backpressured_new_source_byte_count: plan.frame_input.backpressured_new_source_byte_count,
        rejected_new_source_count: plan.frame_input.rejected_new_source_count,
        rejected_new_source_byte_count: plan.frame_input.rejected_new_source_byte_count,
        submission_report: plan.submission.submission_report(),
        completed_retried_source_count: plan.frame_outcome.completed_retried_source_count,
        completed_new_source_count: plan.frame_outcome.completed_new_source_count,
        blocked_retried_source_count: plan.frame_outcome.blocked_retried_source_count,
        blocked_new_source_count: plan.frame_outcome.blocked_new_source_count,
        next_blocked_glyph_count: plan.frame_outcome.next_blocked_glyphs.len(),
        next_blocked_new_source_count: plan.frame_outcome.deferred_new_source_count,
        unmapped_blocked_source_count: plan.frame_outcome.unmapped_blocked_source_count,
        next_retry_frame_index: plan.frame_outcome.next_retry_frame_index,
    }
}
