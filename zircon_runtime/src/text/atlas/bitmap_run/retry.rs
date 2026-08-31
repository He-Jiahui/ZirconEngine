use std::collections::HashSet;

use super::failure::GlyphAtlasBitmapQueuedGlyph;
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRetryBackpressurePolicy {
    pub(crate) max_due_retry_sources_per_frame: Option<usize>,
    pub(crate) max_due_retry_source_bytes_per_frame: Option<usize>,
    pub(crate) max_new_sources_per_frame: Option<usize>,
    pub(crate) max_new_source_bytes_per_frame: Option<usize>,
    pub(crate) max_queued_blocked_glyphs: Option<usize>,
    pub(crate) max_queued_blocked_source_bytes: Option<usize>,
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
            max_due_retry_source_bytes_per_frame: None,
            max_new_sources_per_frame: None,
            max_new_source_bytes_per_frame: None,
            max_queued_blocked_glyphs: None,
            max_queued_blocked_source_bytes: None,
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
    pub(crate) due_retry_source_byte_count: usize,
    pub(crate) deferred_retry_count: usize,
    pub(crate) deferred_retry_source_byte_count: usize,
    pub(crate) backpressured_retry_count: usize,
    pub(crate) backpressured_retry_source_byte_count: usize,
    pub(crate) rejected_retry_source_count: usize,
    pub(crate) rejected_retry_source_byte_count: usize,
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
    glyph_atlas_bitmap_retry_frame_input_with_backpressure_and_new_source_budget_predicate(
        blocked_glyphs,
        frame_sources,
        frame_index,
        backpressure_policy,
        |_| true,
    )
}

pub(crate) fn glyph_atlas_bitmap_retry_frame_input_with_backpressure_and_new_source_budget_predicate<
    R,
    S,
    P,
>(
    blocked_glyphs: R,
    frame_sources: S,
    frame_index: u64,
    backpressure_policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    mut requires_new_source_budget: P,
) -> GlyphAtlasBitmapRetryFrameInput
where
    R: IntoIterator<Item = GlyphAtlasBitmapQueuedGlyph>,
    S: IntoIterator<Item = GlyphAtlasBitmapSource>,
    P: FnMut(GlyphAtlasBitmapSource) -> bool,
{
    let retry_plan = glyph_atlas_bitmap_retry_plan_with_backpressure(
        blocked_glyphs,
        frame_index,
        backpressure_policy,
    );
    let frame_sources = frame_sources.into_iter();
    let (minimum_frame_source_count, _) = frame_sources.size_hint();
    let retry_source_count = retry_plan.retry_glyphs.len();
    let source_capacity = retry_source_count.saturating_add(minimum_frame_source_count);
    let mut scheduled_raster_keys = HashSet::with_capacity(retry_source_count);
    scheduled_raster_keys.extend(
        retry_plan
            .retry_glyphs
            .iter()
            .filter_map(|glyph| glyph.source.raster_key),
    );
    let mut input = GlyphAtlasBitmapRetryFrameInput {
        sources: Vec::with_capacity(source_capacity),
        source_origins: Vec::with_capacity(source_capacity),
        deferred_glyphs: retry_plan.deferred_glyphs,
        retried_source_count: retry_plan.due_retry_count,
        retried_source_byte_count: retry_plan.due_retry_source_byte_count,
        deferred_retry_count: retry_plan.deferred_retry_count,
        deferred_retry_source_byte_count: retry_plan.deferred_retry_source_byte_count,
        backpressured_retry_count: retry_plan.backpressured_retry_count,
        backpressured_retry_source_byte_count: retry_plan.backpressured_retry_source_byte_count,
        rejected_retry_source_count: retry_plan.rejected_retry_source_count,
        rejected_retry_source_byte_count: retry_plan.rejected_retry_source_byte_count,
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

    for (source_index, source) in frame_sources.enumerate() {
        let has_scheduled_duplicate = source
            .raster_key
            .is_some_and(|key| scheduled_raster_keys.contains(&key));
        let is_budgeted_new_source = requires_new_source_budget(source) && !has_scheduled_duplicate;
        if is_budgeted_new_source
            && source_exceeds_byte_budget(
                backpressure_policy.max_new_source_bytes_per_frame,
                source.source_byte_len,
            )
        {
            input.rejected_new_source_count += 1;
            input.rejected_new_source_byte_count = input
                .rejected_new_source_byte_count
                .saturating_add(source.source_byte_len);
            continue;
        }
        if is_budgeted_new_source
            && !new_source_budget_allows(
                backpressure_policy,
                input.budgeted_new_source_count,
                input.budgeted_new_source_byte_count,
                source.source_byte_len,
            )
        {
            let retry_frame_index =
                deferred_new_source_retry_frame_index(frame_index, backpressure_policy);
            input.backpressured_new_source_count += 1;
            input.backpressured_new_source_byte_count = input
                .backpressured_new_source_byte_count
                .saturating_add(source.source_byte_len);
            input.deferred_new_source_count += 1;
            input.deferred_new_source_byte_count = input
                .deferred_new_source_byte_count
                .saturating_add(source.source_byte_len);
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
        input.new_source_byte_count = input
            .new_source_byte_count
            .saturating_add(source.source_byte_len);
        if is_budgeted_new_source {
            if let Some(key) = source.raster_key {
                scheduled_raster_keys.insert(key);
            }
            input.budgeted_new_source_count += 1;
            input.budgeted_new_source_byte_count = input
                .budgeted_new_source_byte_count
                .saturating_add(source.source_byte_len);
        }
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
        if source_exceeds_byte_budget(
            backpressure_policy.max_due_retry_source_bytes_per_frame,
            glyph.source.source_byte_len,
        ) {
            plan.rejected_retry_source_count += 1;
            plan.rejected_retry_source_byte_count = plan
                .rejected_retry_source_byte_count
                .saturating_add(glyph.source.source_byte_len);
            continue;
        }
        if glyph.retry_frame_index <= frame_index {
            if retry_budget_allows(
                backpressure_policy,
                plan.retry_glyphs.len(),
                plan.due_retry_source_byte_count,
                glyph.source.source_byte_len,
            ) {
                plan.due_retry_source_byte_count = plan
                    .due_retry_source_byte_count
                    .saturating_add(glyph.source.source_byte_len);
                plan.retry_glyphs.push(glyph);
                continue;
            }

            plan.backpressured_retry_count += 1;
            plan.backpressured_retry_source_byte_count = plan
                .backpressured_retry_source_byte_count
                .saturating_add(glyph.source.source_byte_len);
            glyph.retry_frame_index = backpressured_retry_frame_index(
                glyph.retry_frame_index,
                frame_index,
                backpressure_policy,
            );
        }

        update_next_retry_frame_index(&mut plan.next_retry_frame_index, glyph.retry_frame_index);
        plan.deferred_retry_source_byte_count = plan
            .deferred_retry_source_byte_count
            .saturating_add(glyph.source.source_byte_len);
        plan.deferred_glyphs.push(glyph);
    }

    plan.due_retry_count = plan.retry_glyphs.len();
    plan.deferred_retry_count = plan.deferred_glyphs.len();
    plan
}

fn retry_budget_allows(
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    scheduled_retry_count: usize,
    scheduled_retry_source_byte_count: usize,
    source_byte_len: usize,
) -> bool {
    count_budget_allows(
        policy.max_due_retry_sources_per_frame,
        scheduled_retry_count,
    ) && byte_budget_allows(
        policy.max_due_retry_source_bytes_per_frame,
        scheduled_retry_source_byte_count,
        source_byte_len,
    )
}

fn new_source_budget_allows(
    policy: GlyphAtlasBitmapRetryBackpressurePolicy,
    scheduled_new_source_count: usize,
    scheduled_new_source_byte_count: usize,
    source_byte_len: usize,
) -> bool {
    count_budget_allows(policy.max_new_sources_per_frame, scheduled_new_source_count)
        && byte_budget_allows(
            policy.max_new_source_bytes_per_frame,
            scheduled_new_source_byte_count,
            source_byte_len,
        )
}

fn count_budget_allows(max_count: Option<usize>, scheduled_count: usize) -> bool {
    max_count.is_none_or(|max_count| scheduled_count < max_count)
}

fn byte_budget_allows(
    max_byte_count: Option<usize>,
    scheduled_byte_count: usize,
    source_byte_len: usize,
) -> bool {
    max_byte_count.is_none_or(|max_byte_count| {
        scheduled_byte_count.saturating_add(source_byte_len) <= max_byte_count
    })
}

fn source_exceeds_byte_budget(max_byte_count: Option<usize>, source_byte_len: usize) -> bool {
    max_byte_count.is_some_and(|max_byte_count| source_byte_len > max_byte_count)
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

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830cw_bitmap_retry_capacity_source_contract() {
        let source = include_str!("retry.rs");
        let input = source
            .split("let frame_sources = frame_sources.into_iter()")
            .nth(1)
            .expect("bitmap retry input capacity implementation")
            .split("for glyph in retry_plan.retry_glyphs")
            .next()
            .expect("bounded bitmap retry input capacity implementation");

        assert!(input.contains("HashSet::with_capacity(retry_source_count)"));
        assert!(input.contains("Vec::with_capacity(source_capacity)"));
        assert!(input.contains("minimum_frame_source_count"));
        assert!(!input.contains("collect::<HashSet"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260830cw_runtime_bitmap_retry_capacity_p95() {
        fn measure(keys: &[Option<u64>], reserve: bool) -> u128 {
            let started = std::time::Instant::now();
            for _ in 0..32 {
                let mut scheduled = if reserve {
                    std::collections::HashSet::with_capacity(keys.len())
                } else {
                    std::collections::HashSet::new()
                };
                scheduled.extend(std::hint::black_box(keys).iter().filter_map(|key| *key));
                let mut sources = if reserve {
                    Vec::with_capacity(keys.len().saturating_mul(2))
                } else {
                    Vec::new()
                };
                let mut origins = if reserve {
                    Vec::with_capacity(keys.len().saturating_mul(2))
                } else {
                    Vec::new()
                };
                for (index, key) in keys.iter().enumerate() {
                    if key.is_some() {
                        sources.push(index as u64);
                        origins.push(index);
                    }
                }
                std::hint::black_box((scheduled, sources, origins));
            }
            started.elapsed().as_nanos()
        }

        let keys = (0..32_768_u64)
            .map(|index| (index % 8 != 0).then_some(index % 16_384))
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for sample_index in 0..17 {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(&keys, false));
                optimized_samples.push(measure(&keys, true));
            } else {
                optimized_samples.push(measure(&keys, true));
                legacy_samples.push(measure(&keys, false));
            }
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "RUNTIME399_BITMAP_RETRY_CAPACITY_BENCH_V1 retry_sources={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            keys.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "capacity-sized bitmap retry P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }
}
