use crate::core::math::UVec2;

use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    GlyphAtlasBitmapQueuedGlyph, GlyphAtlasBitmapRetryBackpressurePolicy, GlyphAtlasBitmapSource,
    GlyphAtlasSet,
};
use super::retry::{
    GlyphAtlasBitmapRetryFrameSubmissionPlan,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_backpressure_and_padding,
    glyph_atlas_bitmap_retry_frame_submission_plan_with_padding,
};

/// Cross-frame blocked glyph queue owned below the renderer root.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameState {
    blocked_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pending_invalidated_blocked_glyph_count: usize,
    pending_queue_overflow_blocked_glyph_count: usize,
    pending_queue_overflow_blocked_source_byte_count: usize,
}

/// Compact state telemetry for renderer frame-loop handoff.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameStateReport {
    pub(crate) queued_blocked_glyph_count: usize,
    pub(crate) queued_blocked_source_byte_count: usize,
    pub(crate) invalidated_blocked_glyph_count: usize,
    pub(crate) queue_overflow_blocked_glyph_count: usize,
    pub(crate) queue_overflow_blocked_source_byte_count: usize,
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

    pub(crate) fn discard_all_for_face_invalidation(&mut self) {
        let invalidated_count = self.blocked_glyphs.len();
        self.blocked_glyphs.clear();
        self.pending_invalidated_blocked_glyph_count = self
            .pending_invalidated_blocked_glyph_count
            .saturating_add(invalidated_count);
    }

    pub(crate) fn report(&self) -> GlyphAtlasBitmapRetryFrameStateReport {
        GlyphAtlasBitmapRetryFrameStateReport {
            queued_blocked_glyph_count: self.queued_blocked_glyph_count(),
            queued_blocked_source_byte_count: self.queued_blocked_source_byte_count(),
            invalidated_blocked_glyph_count: self.pending_invalidated_blocked_glyph_count,
            queue_overflow_blocked_glyph_count: self.pending_queue_overflow_blocked_glyph_count,
            queue_overflow_blocked_source_byte_count: self
                .pending_queue_overflow_blocked_source_byte_count,
            next_retry_frame_index: self.next_retry_frame_index(),
        }
    }

    pub(crate) fn take_report(&mut self) -> GlyphAtlasBitmapRetryFrameStateReport {
        let report = self.report();
        self.pending_invalidated_blocked_glyph_count = 0;
        self.pending_queue_overflow_blocked_glyph_count = 0;
        self.pending_queue_overflow_blocked_source_byte_count = 0;
        report
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submission_plan_with_atlas_backpressure_and_padding<S>(
        &self,
        atlas: GlyphAtlasSet,
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
        glyph_atlas_bitmap_retry_frame_submission_plan_with_atlas_backpressure_and_padding(
            atlas,
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
        self.take_report()
    }

    pub(crate) fn apply_submission_plan_with_backpressure(
        &mut self,
        plan: &GlyphAtlasBitmapRetryFrameSubmissionPlan,
        backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    ) -> GlyphAtlasBitmapRetryFrameStateReport {
        let mut retained = Vec::with_capacity(plan.frame_outcome.next_blocked_glyphs.len());
        let mut retained_source_byte_count = 0usize;
        for glyph in plan.frame_outcome.next_blocked_glyphs.iter().copied() {
            if retry_queue_budget_allows(
                backpressure_policy,
                retained.len(),
                retained_source_byte_count,
                glyph.source.source_byte_len,
            ) {
                retained_source_byte_count =
                    retained_source_byte_count.saturating_add(glyph.source.source_byte_len);
                retained.push(glyph);
            } else {
                self.pending_queue_overflow_blocked_glyph_count = self
                    .pending_queue_overflow_blocked_glyph_count
                    .saturating_add(1);
                self.pending_queue_overflow_blocked_source_byte_count = self
                    .pending_queue_overflow_blocked_source_byte_count
                    .saturating_add(glyph.source.source_byte_len);
            }
        }
        self.blocked_glyphs = retained;
        self.take_report()
    }

    fn queued_blocked_source_byte_count(&self) -> usize {
        self.blocked_glyphs
            .iter()
            .fold(0usize, |byte_count, glyph| {
                byte_count.saturating_add(glyph.source.source_byte_len)
            })
    }
}

impl GlyphAtlasBitmapRetryFrameStateReport {
    pub(crate) fn has_queued_retry_work(self) -> bool {
        self.queued_blocked_glyph_count > 0
    }

    pub(crate) fn has_queue_overflow(self) -> bool {
        self.queue_overflow_blocked_glyph_count > 0
    }
}

fn retry_queue_budget_allows(
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    queued_glyph_count: usize,
    queued_source_byte_count: usize,
    source_byte_len: usize,
) -> bool {
    policy
        .max_queued_blocked_glyphs
        .is_none_or(|max_glyph_count| queued_glyph_count < max_glyph_count)
        && policy
            .max_queued_blocked_source_bytes
            .is_none_or(|max_byte_count| {
                queued_source_byte_count.saturating_add(source_byte_len) <= max_byte_count
            })
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
