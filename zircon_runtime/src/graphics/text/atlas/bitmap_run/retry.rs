use super::failure::GlyphAtlasBitmapQueuedGlyph;
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRetryBackpressurePolicy {
    pub(crate) max_due_retry_sources_per_frame: Option<usize>,
    pub(crate) max_new_sources_per_frame: Option<usize>,
    pub(crate) defer_excess_by_frames: u64,
}

impl Default for GlyphAtlasBitmapRetryBackpressurePolicy {
    fn default() -> Self {
        Self::unlimited()
    }
}

impl GlyphAtlasBitmapRetryBackpressurePolicy {
    pub(crate) fn unlimited() -> Self {
        Self {
            max_due_retry_sources_per_frame: None,
            max_new_sources_per_frame: None,
            defer_excess_by_frames: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapRetrySourceOrigin {
    Retried {
        source_index: usize,
        retry_frame_index: u64,
    },
    New {
        source_index: usize,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameInput {
    pub(crate) sources: Vec<GlyphAtlasBitmapSource>,
    pub(crate) source_origins: Vec<GlyphAtlasBitmapRetrySourceOrigin>,
    pub(crate) deferred_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) deferred_new_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) retried_source_count: usize,
    pub(crate) new_source_count: usize,
    pub(crate) deferred_retry_count: usize,
    pub(crate) backpressured_retry_count: usize,
    pub(crate) deferred_new_source_count: usize,
    pub(crate) backpressured_new_source_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryFrameOutcome {
    pub(crate) next_blocked_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) completed_retried_source_count: usize,
    pub(crate) completed_new_source_count: usize,
    pub(crate) blocked_retried_source_count: usize,
    pub(crate) blocked_new_source_count: usize,
    pub(crate) deferred_retry_count: usize,
    pub(crate) deferred_new_source_count: usize,
    pub(crate) unmapped_blocked_source_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRetryPlan {
    pub(crate) retry_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) deferred_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    pub(crate) due_retry_count: usize,
    pub(crate) deferred_retry_count: usize,
    pub(crate) backpressured_retry_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
}

impl GlyphAtlasBitmapRetryPlan {
    pub(crate) fn retry_sources(&self) -> impl Iterator<Item = GlyphAtlasBitmapSource> + '_ {
        self.retry_glyphs.iter().map(|glyph| glyph.source)
    }
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_input<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    frame_index: u64,
) -> GlyphAtlasBitmapRetryFrameInput
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_retry_frame_input_with_backpressure(
        blocked_glyphs,
        frame_sources,
        frame_index,
        GlyphAtlasBitmapRetryBackpressurePolicy::unlimited(),
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_input_with_backpressure<R, S>(
    blocked_glyphs: R,
    frame_sources: S,
    frame_index: u64,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
) -> GlyphAtlasBitmapRetryFrameInput
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let retry_plan = glyph_atlas_bitmap_retry_plan_with_backpressure(
        blocked_glyphs,
        frame_index,
        backpressure_policy,
    );
    let mut input = GlyphAtlasBitmapRetryFrameInput {
        deferred_glyphs: retry_plan.deferred_glyphs,
        retried_source_count: retry_plan.due_retry_count,
        deferred_retry_count: retry_plan.deferred_retry_count,
        backpressured_retry_count: retry_plan.backpressured_retry_count,
        next_retry_frame_index: retry_plan.next_retry_frame_index,
        ..GlyphAtlasBitmapRetryFrameInput::default()
    };

    for glyph in retry_plan.retry_glyphs {
        input.sources.push(glyph.source);
        input
            .source_origins
            .push(GlyphAtlasBitmapRetrySourceOrigin::Retried {
                source_index: glyph.source_index,
                retry_frame_index: glyph.retry_frame_index,
            });
    }

    for (source_index, source) in frame_sources.into_iter().enumerate() {
        if !new_source_budget_allows(backpressure_policy, input.new_source_count) {
            let retry_frame_index =
                deferred_new_source_retry_frame_index(frame_index, backpressure_policy);
            input.backpressured_new_source_count += 1;
            input.deferred_new_source_count += 1;
            update_next_retry_frame_index(&mut input.next_retry_frame_index, retry_frame_index);
            input.deferred_new_glyphs.push(GlyphAtlasBitmapQueuedGlyph {
                source_index,
                source,
                retry_frame_index,
            });
            continue;
        }

        input.sources.push(source);
        input
            .source_origins
            .push(GlyphAtlasBitmapRetrySourceOrigin::New { source_index });
        input.new_source_count += 1;
    }

    input
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_outcome(
    input: &GlyphAtlasBitmapRetryFrameInput,
    run_plan: &GlyphAtlasBitmapRunPlan,
) -> GlyphAtlasBitmapRetryFrameOutcome {
    let mut outcome = GlyphAtlasBitmapRetryFrameOutcome {
        next_blocked_glyphs: input.deferred_glyphs.clone(),
        deferred_retry_count: input.deferred_glyphs.len(),
        deferred_new_source_count: input.deferred_new_glyphs.len(),
        next_retry_frame_index: input.next_retry_frame_index,
        ..GlyphAtlasBitmapRetryFrameOutcome::default()
    };
    outcome
        .next_blocked_glyphs
        .extend(input.deferred_new_glyphs.iter().copied());

    for glyph in &run_plan.glyphs {
        match input.source_origins.get(glyph.source_index).copied() {
            Some(GlyphAtlasBitmapRetrySourceOrigin::Retried { .. }) => {
                outcome.completed_retried_source_count += 1;
            }
            Some(GlyphAtlasBitmapRetrySourceOrigin::New { .. }) => {
                outcome.completed_new_source_count += 1;
            }
            None => {}
        }
    }

    for blocked in &run_plan.blocked_glyphs {
        let mut queued = *blocked;
        match input.source_origins.get(blocked.source_index).copied() {
            Some(GlyphAtlasBitmapRetrySourceOrigin::Retried { source_index, .. }) => {
                queued.source_index = source_index;
                outcome.blocked_retried_source_count += 1;
            }
            Some(GlyphAtlasBitmapRetrySourceOrigin::New { source_index }) => {
                queued.source_index = source_index;
                outcome.blocked_new_source_count += 1;
            }
            None => {
                outcome.unmapped_blocked_source_count += 1;
            }
        }
        update_next_retry_frame_index(
            &mut outcome.next_retry_frame_index,
            queued.retry_frame_index,
        );
        outcome.next_blocked_glyphs.push(queued);
    }

    outcome
}

pub(crate) fn glyph_atlas_bitmap_retry_plan<I>(
    blocked_glyphs: I,
    frame_index: u64,
) -> GlyphAtlasBitmapRetryPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
{
    glyph_atlas_bitmap_retry_plan_with_backpressure(
        blocked_glyphs,
        frame_index,
        GlyphAtlasBitmapRetryBackpressurePolicy::unlimited(),
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_plan_with_backpressure<I>(
    blocked_glyphs: I,
    frame_index: u64,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
) -> GlyphAtlasBitmapRetryPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
{
    let mut plan = GlyphAtlasBitmapRetryPlan::default();

    for mut glyph in blocked_glyphs {
        if glyph.retry_frame_index <= frame_index {
            if retry_budget_allows(backpressure_policy, plan.retry_glyphs.len()) {
                plan.retry_glyphs.push(glyph);
                continue;
            }

            plan.backpressured_retry_count += 1;
            glyph.retry_frame_index = backpressured_retry_frame_index(
                glyph.retry_frame_index,
                frame_index,
                backpressure_policy,
            );
        }

        update_next_retry_frame_index(&mut plan.next_retry_frame_index, glyph.retry_frame_index);
        plan.deferred_glyphs.push(glyph);
    }

    plan.due_retry_count = plan.retry_glyphs.len();
    plan.deferred_retry_count = plan.deferred_glyphs.len();
    plan
}

fn retry_budget_allows(
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    scheduled_retry_count: usize,
) -> bool {
    match policy.max_due_retry_sources_per_frame {
        Some(max_retry_count) => scheduled_retry_count < max_retry_count,
        None => true,
    }
}

fn new_source_budget_allows(
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    scheduled_new_source_count: usize,
) -> bool {
    match policy.max_new_sources_per_frame {
        Some(max_new_source_count) => scheduled_new_source_count < max_new_source_count,
        None => true,
    }
}

fn backpressured_retry_frame_index(
    previous_retry_frame_index: u64,
    frame_index: u64,
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
) -> u64 {
    let defer_frames = policy.defer_excess_by_frames.max(1);
    previous_retry_frame_index.max(frame_index.saturating_add(defer_frames))
}

fn deferred_new_source_retry_frame_index(
    frame_index: u64,
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
) -> u64 {
    frame_index.saturating_add(policy.defer_excess_by_frames.max(1))
}

fn update_next_retry_frame_index(next_retry_frame_index: &mut Option<u64>, retry_frame_index: u64) {
    *next_retry_frame_index = next_retry_frame_index.map_or(Some(retry_frame_index), |next| {
        Some(next.min(retry_frame_index))
    });
}
