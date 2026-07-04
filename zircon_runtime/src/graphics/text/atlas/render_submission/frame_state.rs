use crate::core::math::UVec2;

use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasBitmapQueuedGlyph, GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapSource,
};
use super::retry::{
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding,
    GlyphAtlasBitmapRetryFrameSubmissionPlan,
};

/// Cross-frame blocked glyph queue owned below the renderer root.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameState {
    blocked_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
}

/// Compact state telemetry for renderer frame-loop handoff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameStateReport {
    pub(crate) queued_blocked_glyph_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
}

impl GlyphAtlasBitmapRetryFrameState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_blocked_glyphs<I>(blocked_glyphs: I) -> Self
    where
        I: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    {
        let mut state = Self::new();
        state.replace_blocked_glyphs(blocked_glyphs);
        state
    }

    pub(crate) fn queued_blocked_glyphs(&self) -> &[GlyphAtlasBitmapQueuedGlyph] {
        &self.blocked_glyphs
    }

    pub(crate) fn queued_blocked_glyph_count(&self) -> usize {
        self.blocked_glyphs.len()
    }

    pub(crate) fn next_retry_frame_index(&self) -> Option<u64> {
        next_retry_frame_index(self.blocked_glyphs.iter())
    }

    pub(crate) fn replace_blocked_glyphs<I>(&mut self, blocked_glyphs: I)
    where
        I: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    {
        self.blocked_glyphs = blocked_glyphs.into_iter().collect();
    }

    pub(crate) fn report(&self) -> GlyphAtlasBitmapRetryFrameStateReport {
        GlyphAtlasBitmapRetryFrameStateReport {
            queued_blocked_glyph_count: self.queued_blocked_glyph_count(),
            next_retry_frame_index: self.next_retry_frame_index(),
        }
    }

    pub(crate) fn submission_plan_with_padding<S>(
        &self,
        frame_sources: S,
        page_size: UVec2,
        frame_index: u64,
        max_pages_per_format: usize,
        padding_px: u32,
        viewport_size: UVec2,
        clip_rect: GlyphAtlasScreenRect,
    ) -> GlyphAtlasBitmapRetryFrameSubmissionPlan
    where
        S: IntoIterator<Item = GlyphAtlasBitmapSource>,
    {
        glyph_atlas_bitmap_retry_frame_submission_plan_with_padding(
            self.blocked_glyphs.iter().copied(),
            frame_sources,
            page_size,
            frame_index,
            max_pages_per_format,
            padding_px,
            viewport_size,
            clip_rect,
        )
    }

    pub(crate) fn submission_plan_with_backpressure_and_padding<S>(
        &self,
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
        S: IntoIterator<Item = GlyphAtlasBitmapSource>,
    {
        glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding(
            self.blocked_glyphs.iter().copied(),
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

    pub(crate) fn apply_submission_plan(
        &mut self,
        plan: &GlyphAtlasBitmapRetryFrameSubmissionPlan,
    ) -> GlyphAtlasBitmapRetryFrameStateReport {
        self.replace_blocked_glyphs(plan.frame_outcome.next_blocked_glyphs.iter().copied());
        self.report()
    }
}

impl GlyphAtlasBitmapRetryFrameStateReport {
    pub(crate) fn has_queued_retry_work(self) -> bool {
        self.queued_blocked_glyph_count > 0
    }
}

fn next_retry_frame_index<'a, I>(blocked_glyphs: I) -> Option<u64>
where
    I: IntoIterator<Item = &'a GlyphAtlasBitmapQueuedGlyph>,
{
    blocked_glyphs
        .into_iter()
        .map(|glyph| glyph.retry_frame_index)
        .min()
}
