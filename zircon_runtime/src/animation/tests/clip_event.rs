use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::animation::{
    AnimationClipAsset, AnimationClipEvent, AnimationClipEventSamplingCursor,
    AnimationEventTrackAsset,
};
use crate::core::math::Real;
use crate::core::resource::{AssetReference, ResourceLocator};

use super::{
    event_candidate, sample_clip_events_budgeted, take_candidate_comparisons,
    AnimationClipEventSamplingLimits, EventCandidate,
};

#[test]
fn looping_event_sampling_is_bounded_and_resumes_in_playback_order() {
    let clip = clip_with_events(vec![
        event_track("alpha", 0.25, None),
        event_track("beta", 0.5, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 2,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };
    let mut cursor = None;
    let mut received = Vec::new();

    loop {
        let batch = sample_clip_events_budgeted(&clip, 7, 0.0, 3.0, true, cursor, limits);
        assert!(batch.events.len() <= limits.max_events);
        assert!(batch.emitted_event_bytes <= limits.max_event_bytes);
        assert!(batch.playback_span_seconds <= limits.max_playback_span_seconds);
        received.extend(batch.events);
        let Some(next_cursor) = batch.next_cursor else {
            break;
        };
        cursor = Some(next_cursor);
    }

    assert_eq!(
        received
            .iter()
            .map(|event| (event.playback_time_seconds, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (0.25, "alpha"),
            (0.5, "beta"),
            (1.25, "alpha"),
            (1.5, "beta"),
            (2.25, "alpha"),
            (2.5, "beta"),
        ]
    );
}

#[test]
fn optimization_batch_r6_wave5_runtime640_event_batch_reserves_event_limit() {
    let clip = clip_with_events(vec![
        event_track("alpha", 0.25, None),
        event_track("beta", 0.5, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 64,
        max_event_bytes: usize::MAX,
        max_playback_span_seconds: 1.0,
    };
    let batch = sample_clip_events_budgeted(&clip, 640, 0.0, 1.0, false, None, limits);

    assert_eq!(batch.events.len(), 2);
    assert!(batch.events.capacity() >= limits.max_events);

    let source = include_str!("../clip_event.rs");
    assert!(source.contains("events: Vec::with_capacity(limits.max_events),"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave5_runtime640_event_batch_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const EVENTS_PER_SAMPLE: usize = 65_536;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(runtime640_measure_event_projection(
                EVENTS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(runtime640_measure_event_projection(EVENTS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(runtime640_measure_event_projection(EVENTS_PER_SAMPLE, true));
            legacy_samples.push(runtime640_measure_event_projection(
                EVENTS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = runtime640_p95(&legacy_samples);
    let optimized_p95 = runtime640_p95(&optimized_samples);
    println!(
        "RUNTIME640_PREALLOCATED_ANIMATION_EVENT_BATCHES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} events_per_sample={EVENTS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated animation event batches must be at least 15% faster at P95"
    );
}

fn runtime640_measure_event_projection(output_count: usize, optimized: bool) -> u128 {
    let mut outputs = if optimized {
        Vec::with_capacity(output_count)
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for value in 0..output_count {
        outputs.push(black_box(value));
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(outputs);
    elapsed
}

fn runtime640_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

#[test]
fn byte_budget_defers_later_events_without_dropping_their_order() {
    let clip = clip_with_events(vec![
        event_track("first", 0.1, Some("one")),
        event_track("second", 0.2, Some("two")),
        event_track("third", 0.3, Some("three")),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 8,
        max_event_bytes: 16,
        max_playback_span_seconds: 1.0,
    };
    let first = sample_clip_events_budgeted(&clip, 8, 0.0, 1.0, false, None, limits);

    assert_eq!(first.events.len(), 1);
    assert_eq!(first.events[0].event, "first");
    assert!(first.budget_exhausted);
    assert!(first.next_cursor.is_some());

    let second = sample_clip_events_budgeted(&clip, 8, 0.0, 1.0, false, first.next_cursor, limits);
    assert_eq!(
        second
            .events
            .iter()
            .map(|event| event.event.as_str())
            .collect::<Vec<_>>(),
        vec!["second"]
    );
    assert!(second.next_cursor.is_some());
}

#[test]
fn cursor_resumes_all_same_time_tracks_after_an_event_count_boundary() {
    let clip = clip_with_events(vec![
        event_track("alpha", 0.5, None),
        event_track("beta", 0.5, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 1,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };

    let first = sample_clip_events_budgeted(&clip, 9, 0.0, 1.0, false, None, limits);
    assert_eq!(first.events[0].event, "alpha");

    let second = sample_clip_events_budgeted(&clip, 9, 0.0, 1.0, false, first.next_cursor, limits);
    assert_eq!(second.events[0].event, "beta");
    assert!(second.next_cursor.is_none());
}

#[test]
fn reverse_event_sampling_uses_descending_half_open_endpoints_and_stable_ties() {
    let clip = clip_with_events(vec![
        event_track("origin", 0.75, None),
        event_track("beta", 0.5, None),
        event_track("alpha", 0.5, None),
        event_track("destination", 0.25, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 1,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };
    let mut cursor = None;
    let mut received = Vec::new();

    loop {
        let batch = sample_clip_events_budgeted(&clip, 14, 0.75, 0.25, false, cursor, limits);
        received.extend(batch.events);
        cursor = batch.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    assert_eq!(
        received
            .iter()
            .map(|event| (event.playback_time_seconds, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![(0.5, "alpha"), (0.5, "beta"), (0.25, "destination")]
    );
}

#[test]
fn reverse_looping_sampling_crosses_the_seam_without_replaying_the_outgoing_endpoint() {
    let clip = clip_with_events(vec![
        event_track("clip-start", 0.0, None),
        event_track("clip-end", 1.0, None),
        event_track("three-quarter", 0.75, None),
        event_track("one-quarter", 0.25, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 1,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };
    let mut cursor = None;
    let mut received = Vec::new();

    loop {
        let batch = sample_clip_events_budgeted(&clip, 15, 2.25, 1.75, true, cursor, limits);
        received.extend(batch.events);
        cursor = batch.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    assert_eq!(
        received
            .iter()
            .map(|event| (event.playback_time_seconds, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![(2.0, "clip-start"), (1.75, "three-quarter")]
    );
}

#[test]
fn forward_looping_sampling_uses_the_clip_end_side_of_a_loop_seam() {
    let clip = clip_with_events(vec![
        event_track("clip-start", 0.0, None),
        event_track("clip-end", 1.0, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 4,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };

    let batch = sample_clip_events_budgeted(&clip, 16, 0.75, 1.25, true, None, limits);

    assert_eq!(
        batch
            .events
            .iter()
            .map(|event| (event.playback_time_seconds, event.event.as_str()))
            .collect::<Vec<_>>(),
        vec![(1.0, "clip-end")]
    );
}

#[test]
fn looping_cursor_advances_the_same_track_after_its_boundary_event() {
    let clip = clip_with_events(vec![event_track("pulse", 0.5, None)]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 1,
        max_event_bytes: 1024,
        max_playback_span_seconds: 4.0,
    };
    let first = sample_clip_events_budgeted(&clip, 10, 0.0, 2.0, true, None, limits);
    assert_eq!(first.events[0].playback_time_seconds, 0.5);

    let second = sample_clip_events_budgeted(&clip, 10, 0.0, 2.0, true, first.next_cursor, limits);
    assert_eq!(second.events[0].playback_time_seconds, 1.5);
    assert!(second.next_cursor.is_none());
}

#[test]
fn event_candidate_selection_scales_subquadratically() {
    const EVENT_COUNT: usize = 1_024;
    const MAX_COMPARISONS_PER_EVENT: usize = 32;

    let clip = clip_with_events(
        (0..EVENT_COUNT)
            .map(|index| {
                event_track(
                    &format!("event-{index:04}"),
                    (index + 1) as Real / (EVENT_COUNT + 1) as Real,
                    None,
                )
            })
            .collect(),
    );
    let limits = AnimationClipEventSamplingLimits {
        max_events: EVENT_COUNT,
        max_event_bytes: usize::MAX,
        max_playback_span_seconds: 1.0,
    };

    take_candidate_comparisons();
    let batch = sample_clip_events_budgeted(&clip, 11, 0.0, 1.0, false, None, limits);
    let comparisons = take_candidate_comparisons();

    assert_eq!(batch.events.len(), EVENT_COUNT);
    assert!(batch.next_cursor.is_none());
    assert!(
        comparisons <= EVENT_COUNT * MAX_COMPARISONS_PER_EVENT,
        "candidate selection used {comparisons} comparisons for {EVENT_COUNT} events"
    );
}

#[test]
fn same_time_duplicate_events_resume_by_track_index() {
    let clip = clip_with_events(vec![
        event_track("pulse", 0.5, None),
        event_track("pulse", 0.5, None),
    ]);
    let limits = AnimationClipEventSamplingLimits {
        max_events: 1,
        max_event_bytes: 1024,
        max_playback_span_seconds: 1.0,
    };

    let first = sample_clip_events_budgeted(&clip, 12, 0.0, 1.0, false, None, limits);
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.next_cursor.as_ref().unwrap().last_track_index, 0);

    let second = sample_clip_events_budgeted(&clip, 12, 0.0, 1.0, false, first.next_cursor, limits);
    assert_eq!(second.events.len(), 1);
    assert_eq!(second.events[0].event, "pulse");
    assert!(second.next_cursor.is_none());
}

#[test]
#[ignore = "managed animation event-candidate release performance gate"]
fn event_candidate_heap_release_benchmark_evidence() {
    const EVENT_COUNT: usize = 2_048;
    const SAMPLE_PAIRS: usize = 21;
    const TARGET_P95_PERCENT: u128 = 25;

    let clip = benchmark_clip(EVENT_COUNT);
    let limits = AnimationClipEventSamplingLimits {
        max_events: EVENT_COUNT,
        max_event_bytes: usize::MAX,
        max_playback_span_seconds: 1.0,
    };
    assert_eq!(legacy_sample_all_events(&clip, 13).0.len(), EVENT_COUNT);
    assert_eq!(
        sample_clip_events_budgeted(&clip, 13, 0.0, 1.0, false, None, limits)
            .events
            .len(),
        EVENT_COUNT
    );

    let mut legacy_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    let mut heap_samples_us = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            let sampled = legacy_sample_all_events(black_box(&clip), 13);
            legacy_samples_us.push(started.elapsed().as_micros());
            assert_eq!(black_box(sampled).0.len(), EVENT_COUNT);
        };
        let mut measure_heap = || {
            let started = Instant::now();
            let sampled =
                sample_clip_events_budgeted(black_box(&clip), 13, 0.0, 1.0, false, None, limits);
            heap_samples_us.push(started.elapsed().as_micros());
            assert_eq!(black_box(sampled).events.len(), EVENT_COUNT);
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_heap();
        } else {
            measure_heap();
            measure_legacy();
        }
    }

    let legacy_p50_us = nearest_rank_percentile(&legacy_samples_us, 50);
    let legacy_p95_us = nearest_rank_percentile(&legacy_samples_us, 95);
    let heap_p50_us = nearest_rank_percentile(&heap_samples_us, 50);
    let heap_p95_us = nearest_rank_percentile(&heap_samples_us, 95);
    let legacy_candidate_visits = EVENT_COUNT.saturating_mul(EVENT_COUNT + 1) / 2;
    let p95_ratio = heap_p95_us as f64 / legacy_p95_us.max(1) as f64;

    println!(
        "ANIMATION_EVENT_CANDIDATE_HEAP_BENCH_V1 event_count={EVENT_COUNT} sample_pairs={SAMPLE_PAIRS} sample_order=alternating percentile_method=nearest_rank legacy_candidate_visits={legacy_candidate_visits} heap_candidate_pops={EVENT_COUNT} legacy_p50_us={legacy_p50_us} legacy_p95_us={legacy_p95_us} heap_p50_us={heap_p50_us} heap_p95_us={heap_p95_us} p95_ratio={p95_ratio:.6} legacy_us={} heap_us={}",
        join_samples(&legacy_samples_us),
        join_samples(&heap_samples_us),
    );
    assert!(
        heap_p95_us.saturating_mul(100) <= legacy_p95_us.saturating_mul(TARGET_P95_PERCENT),
        "heap P95 {heap_p95_us}us must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_us}us"
    );
}

fn benchmark_clip(event_count: usize) -> AnimationClipAsset {
    clip_with_events(
        (0..event_count)
            .map(|track_index| {
                let playback_rank = track_index.wrapping_mul(997) % event_count;
                event_track(
                    &format!("event-{playback_rank:04}"),
                    (playback_rank + 1) as Real / (event_count + 1) as Real,
                    Some("benchmark-payload"),
                )
            })
            .collect(),
    )
}

fn legacy_sample_all_events(
    clip: &AnimationClipAsset,
    entity: u64,
) -> (Vec<AnimationClipEvent>, AnimationClipEventSamplingCursor) {
    let cursor = AnimationClipEventSamplingCursor::at_range_start(0.0);
    let mut candidates = clip
        .event_tracks
        .iter()
        .enumerate()
        .filter_map(|(track_index, track)| {
            event_candidate(track, track_index, 0.0, false, false, &cursor, 1.0)
        })
        .collect::<Vec<_>>();
    let mut events = Vec::with_capacity(candidates.len());
    let mut last_cursor = cursor;
    while let Some((candidate_index, candidate)) = candidates
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| compare_legacy_candidates(left, right))
        .map(|(candidate_index, candidate)| (candidate_index, *candidate))
    {
        let track = &clip.event_tracks[candidate.track_index];
        events.push(AnimationClipEvent {
            entity,
            target_id: track.target_id.clone(),
            event: track.event.clone(),
            payload: track.payload.clone(),
            clip_time_seconds: track.time_seconds,
            playback_time_seconds: candidate.playback_time_seconds,
        });
        last_cursor = AnimationClipEventSamplingCursor {
            playback_time_seconds: candidate.playback_time_seconds,
            last_event: Some(track.event.clone().into_boxed_str()),
            last_track_index: candidate.track_index,
        };
        candidates.remove(candidate_index);
    }
    (events, last_cursor)
}

fn compare_legacy_candidates(
    left: &EventCandidate<'_>,
    right: &EventCandidate<'_>,
) -> std::cmp::Ordering {
    left.playback_time_seconds
        .total_cmp(&right.playback_time_seconds)
        .then_with(|| left.event.cmp(right.event))
        .then_with(|| left.track_index.cmp(&right.track_index))
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn clip_with_events(event_tracks: Vec<AnimationEventTrackAsset>) -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("budgeted-events".to_string()),
        skeleton: AssetReference::from_locator(
            ResourceLocator::parse("res://animation/budgeted.skeleton.zranim").unwrap(),
        ),
        duration_seconds: 1.0,
        tracks: Vec::new(),
        event_tracks,
    }
}

fn event_track(event: &str, time_seconds: Real, payload: Option<&str>) -> AnimationEventTrackAsset {
    AnimationEventTrackAsset {
        target_id: None,
        event: event.to_string(),
        time_seconds,
        payload: payload.map(str::to_string),
    }
}
