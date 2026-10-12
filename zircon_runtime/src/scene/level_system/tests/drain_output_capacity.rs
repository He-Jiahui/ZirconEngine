use std::time::Instant;

use crate::core::framework::animation::{
    AnimationClipEvent, AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission,
    AnimationClipEventSampler, AnimationClipEventSamplingBatch, AnimationClipEventSamplingCursor,
    AnimationClipEventSamplingLimits, AnimationClipEventSamplingRange,
    AnimationClipEventSamplingRequest,
};
use crate::core::math::Real;
use crate::core::resource::ResourceId;
use crate::scene::{DefaultLevelManager, LevelSystem};

const BENCHMARK_MARKER: &str = "RUNTIME857_ANIMATION_DRAIN_OUTPUT_CAPACITY_BENCH_V1";
const PROFILE_SAMPLE_COUNT: usize = 31;
const PROFILE_RANGES_PER_SAMPLE: usize = 32;

#[test]
fn runtime857_animation_drain_output_capacity_preserves_order() {
    let values = [3_u32, 5, 8, 13, 21];
    let (output, growth_events) = model_collect(values.iter().copied(), values.len());

    assert_eq!(output, values);
    assert_eq!(growth_events, 0);
}

#[test]
fn runtime857_animation_drain_output_capacity_keeps_empty_batches_zero_capacity() {
    let (output, growth_events) = model_collect(std::iter::empty(), 4_096);

    assert!(output.is_empty());
    assert_eq!(output.capacity(), 0);
    assert_eq!(growth_events, 0);
}

#[test]
fn runtime857_production_drain_reserves_capacity_and_preserves_sample_order() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    enqueue_ranges(
        &level,
        replacement_epoch,
        vec![sample_range(17), sample_range(18)],
    );

    let events = level
        .drain_animation_clip_events(replacement_epoch, &OrderedSampler)
        .expect("current replacement epoch drains the queue");

    assert_eq!(
        events
            .iter()
            .map(|event| (event.entity, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![(17, "first"), (17, "second"), (18, "first"), (18, "second"),]
    );
    assert!(
        events.capacity() >= AnimationClipEventSamplingLimits::default().max_events,
        "a non-empty drain retains the production event budget capacity"
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(0)
    );
    assert!(!level.animation_clip_event_drain_metrics().2);
}

#[test]
fn runtime857_production_empty_drain_keeps_zero_capacity() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();

    let events = level
        .drain_animation_clip_events(replacement_epoch, &OrderedSampler)
        .expect("current replacement epoch drains an empty queue");

    assert!(events.is_empty());
    assert_eq!(events.capacity(), 0);
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(0)
    );
}

#[test]
fn runtime857_production_event_budget_requeues_and_resumes_remaining_ranges() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    enqueue_ranges(
        &level,
        replacement_epoch,
        vec![sample_range(17), sample_range(18)],
    );

    let events = level
        .drain_animation_clip_events(replacement_epoch, &FillEventBudgetSampler)
        .expect("current replacement epoch drains the queue");
    let event_limit = AnimationClipEventSamplingLimits::default().max_events;

    assert_eq!(events.len(), event_limit);
    assert!(events.capacity() >= event_limit);
    assert!(events.iter().all(|event| event.entity == 17));
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(1)
    );
    assert!(level.animation_clip_event_drain_metrics().2);

    let resumed = level
        .drain_animation_clip_events(replacement_epoch, &OrderedSampler)
        .expect("current replacement epoch drains the retained range");
    assert_eq!(
        resumed
            .iter()
            .map(|event| (event.entity, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![(18, "first"), (18, "second")]
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(0)
    );
}

#[test]
fn runtime857_production_drain_finishes_a_resumable_range_before_a_later_direction_change() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let clip_id = ResourceId::from_stable_label("runtime857.animation-direction-change");
    enqueue_ranges(
        &level,
        replacement_epoch,
        vec![
            AnimationClipEventSamplingRange {
                entity: 17,
                clip_id,
                from_time_seconds: 0.0,
                to_time_seconds: 2.0,
                looping: false,
            },
            AnimationClipEventSamplingRange {
                entity: 17,
                clip_id,
                from_time_seconds: 2.0,
                to_time_seconds: 1.0,
                looping: false,
            },
        ],
    );

    let first = level
        .drain_animation_clip_events(replacement_epoch, &ResumableDirectionSampler)
        .expect("current replacement epoch drains the first bounded slice");
    assert_eq!(
        first
            .iter()
            .map(|event| event.event.as_str())
            .collect::<Vec<_>>(),
        vec!["forward-first"]
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(2)
    );

    let resumed = level
        .drain_animation_clip_events(replacement_epoch, &ResumableDirectionSampler)
        .expect("the forward range resumes before the reverse range");
    assert_eq!(
        resumed
            .iter()
            .map(|event| event.event.as_str())
            .collect::<Vec<_>>(),
        vec!["forward-second", "reverse-next"]
    );
    assert_eq!(
        level.animation_clip_event_backlog_len(replacement_epoch),
        Some(0)
    );
}

#[test]
#[ignore = "managed Windows Release production drain latency samples"]
fn runtime857_animation_drain_output_capacity_bench() {
    let level = DefaultLevelManager::default().create_default_level();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let event_limit = AnimationClipEventSamplingLimits::default().max_events;
    let mut raw_samples_nanos = Vec::with_capacity(PROFILE_SAMPLE_COUNT);

    for sample_index in 0..PROFILE_SAMPLE_COUNT {
        let first_entity = (sample_index * PROFILE_RANGES_PER_SAMPLE + 1) as u64;
        let ranges = (0..PROFILE_RANGES_PER_SAMPLE)
            .map(|offset| sample_range(first_entity + offset as u64))
            .collect::<Vec<_>>();
        enqueue_ranges(&level, replacement_epoch, ranges);
        let started = Instant::now();
        let events = level
            .drain_animation_clip_events(replacement_epoch, &OrderedSampler)
            .expect("current replacement epoch drains the queue");
        raw_samples_nanos.push(started.elapsed().as_nanos());

        assert_eq!(events.len(), event_limit);
        assert!(events.capacity() >= event_limit);
        assert_eq!(
            level.animation_clip_event_backlog_len(replacement_epoch),
            Some(0)
        );
    }

    let mut ordered_samples = raw_samples_nanos.clone();
    ordered_samples.sort_unstable();
    println!(
        "{BENCHMARK_MARKER} sample_count={} ranges_per_sample={} event_limit={} raw_nanos={:?} p50_nanos={} p95_nanos={} p99_nanos={}",
        raw_samples_nanos.len(),
        PROFILE_RANGES_PER_SAMPLE,
        event_limit,
        raw_samples_nanos,
        nearest_rank(&ordered_samples, 50),
        nearest_rank(&ordered_samples, 95),
        nearest_rank(&ordered_samples, 99),
    );
}

fn enqueue_ranges(
    level: &LevelSystem,
    replacement_epoch: u64,
    ranges: Vec<AnimationClipEventSamplingRange>,
) {
    let admitted_range_count = ranges.len();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(replacement_epoch, vec![ranges]),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count,
            deferred_range_count: 0,
            rejected_range_count: 0,
        }
    );
}

fn sample_range(entity: u64) -> AnimationClipEventSamplingRange {
    AnimationClipEventSamplingRange {
        entity,
        clip_id: ResourceId::from_stable_label("runtime857.animation-drain-capacity"),
        from_time_seconds: 0.0,
        to_time_seconds: 1.0,
        looping: false,
    }
}

struct OrderedSampler;

impl AnimationClipEventSampler for OrderedSampler {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        let events = vec![
            sampled_event(request.entity, "first", request.from_time_seconds),
            sampled_event(request.entity, "second", request.to_time_seconds),
        ];
        let emitted_event_bytes = events.iter().map(|event| event.event.len()).sum();

        Some(AnimationClipEventSamplingBatch {
            events,
            emitted_event_bytes,
            playback_span_seconds: request.to_time_seconds - request.from_time_seconds,
            ..AnimationClipEventSamplingBatch::default()
        })
    }
}

struct ResumableDirectionSampler;

impl AnimationClipEventSampler for ResumableDirectionSampler {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        if request.from_time_seconds == 0.0 && request.cursor.last_event.is_none() {
            let event = sampled_event(request.entity, "forward-first", 0.5);
            let emitted_event_bytes = event.event.len();
            return Some(AnimationClipEventSamplingBatch {
                events: vec![event],
                next_cursor: Some(AnimationClipEventSamplingCursor {
                    playback_time_seconds: 0.5,
                    last_event: Some("forward-first".into()),
                    last_track_index: 0,
                }),
                emitted_event_bytes,
                playback_span_seconds: 0.5,
                ..AnimationClipEventSamplingBatch::default()
            });
        }

        let (event_name, time_seconds) = if request.from_time_seconds == 0.0 {
            ("forward-second", 1.0)
        } else {
            ("reverse-next", 1.5)
        };
        let event = sampled_event(request.entity, event_name, time_seconds);
        let emitted_event_bytes = event.event.len();
        Some(AnimationClipEventSamplingBatch {
            events: vec![event],
            emitted_event_bytes,
            playback_span_seconds: 0.5,
            ..AnimationClipEventSamplingBatch::default()
        })
    }
}

struct FillEventBudgetSampler;

impl AnimationClipEventSampler for FillEventBudgetSampler {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        let events = (0..request.limits.max_events)
            .map(|index| {
                let time = index as Real / request.limits.max_events as Real;
                sampled_event(request.entity, format!("event-{index}"), time)
            })
            .collect::<Vec<_>>();
        let emitted_event_bytes = events.iter().map(|event| event.event.len()).sum();

        Some(AnimationClipEventSamplingBatch {
            events,
            emitted_event_bytes,
            playback_span_seconds: request.to_time_seconds - request.from_time_seconds,
            ..AnimationClipEventSamplingBatch::default()
        })
    }
}

fn sampled_event(entity: u64, event: impl Into<String>, time: Real) -> AnimationClipEvent {
    AnimationClipEvent {
        entity,
        target_id: None,
        event: event.into(),
        payload: None,
        clip_time_seconds: time,
        playback_time_seconds: time,
    }
}

fn nearest_rank(sorted_samples: &[u128], percentile: usize) -> u128 {
    let rank = (sorted_samples.len() * percentile + 99) / 100;
    sorted_samples[rank - 1]
}

fn model_collect<I>(values: I, max_events: usize) -> (Vec<u32>, usize)
where
    I: IntoIterator<Item = u32>,
{
    let mut values = values.into_iter();
    let Some(first) = values.next() else {
        return (Vec::new(), 0);
    };
    let mut output = Vec::with_capacity(max_events);
    let mut growth_events = 0;
    for value in std::iter::once(first).chain(values) {
        if output.len() == output.capacity() {
            growth_events += 1;
        }
        output.push(value);
    }
    (output, growth_events)
}
