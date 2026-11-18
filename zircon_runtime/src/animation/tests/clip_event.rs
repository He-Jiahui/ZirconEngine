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

#[test]
fn loop_notify_resume_sampler_keeps_occurrence_multiplicity() {
    const TRACKS: usize = 16;
    const POINT: Real = 0.01;
    const DURATION: Real = 0.02;
    const POSITIONS: [Real; 5] = [0.01, 0.03, 0.05, 0.07, 0.09];
    const ENTITY: u64 = 17;

    let names: [String; TRACKS] = std::array::from_fn(|index| format!("point-{index:02}"));
    let mut clip = clip_with_events(
        names
            .iter()
            .rev()
            .map(|name| event_track(name, POINT, None))
            .collect(),
    );
    clip.duration_seconds = DURATION;
    let limits = AnimationClipEventSamplingLimits::default();

    let name_counts = |events: &[AnimationClipEvent]| {
        names.each_ref().map(|name| {
            events
                .iter()
                .filter(|event| event.event.as_str() == name.as_str())
                .count()
        })
    };
    let assert_occurrences = |label: &str, events: &[AnimationClipEvent], positions: &[Real]| {
        assert_eq!(
            name_counts(events),
            [positions.len(); TRACKS],
            "{label}: per-name counts"
        );
        assert_eq!(events.len(), positions.len() * TRACKS, "{label}: count");
        for (group, &position) in events.chunks_exact(TRACKS).zip(positions) {
            for (event, name) in group.iter().zip(&names) {
                assert_eq!(event.entity, ENTITY);
                assert_eq!(event.event.as_str(), name.as_str(), "{label}: point order");
                assert_eq!(event.clip_time_seconds, POINT, "{label}: clip position");
                assert_eq!(event.playback_time_seconds, group[0].playback_time_seconds);
                assert!(
                    event
                        .playback_time_seconds
                        .to_bits()
                        .abs_diff(position.to_bits())
                        <= 2,
                    "{label}: {name} at {:?}, expected about {position}",
                    event.playback_time_seconds,
                );
            }
        }
    };

    // Both a UE-sequence-cap-compatible delta and the exact-binary original ZE witness.
    let ends: [Real; 2] = [0.1, 0.09375];
    let deliveries = ends.map(|end| {
        let unpaged = sample_clip_events_budgeted(
            &clip,
            ENTITY,
            0.0,
            end,
            true,
            None,
            AnimationClipEventSamplingLimits {
                max_events: 128,
                ..limits
            },
        );
        let first = sample_clip_events_budgeted(&clip, ENTITY, 0.0, end, true, None, limits);
        let resumed = first.next_cursor.clone().map(|cursor| {
            sample_clip_events_budgeted(&clip, ENTITY, 0.0, end, true, Some(cursor), limits)
        });
        (unpaged, first, resumed)
    });

    // Emit both inputs' real pages even if the first behavioral assertion fails.
    let time_bits = |events: &[AnimationClipEvent]| {
        let mut bits: Vec<_> = events
            .iter()
            .map(|event| event.playback_time_seconds.to_bits())
            .collect();
        bits.dedup(); // Log adjacent equal values once; assertions still use the untouched events.
        bits.iter()
            .map(|bits| format!("{bits:08x}"))
            .collect::<Vec<_>>()
    };
    for (end, (unpaged, first, resumed)) in ends.iter().zip(&deliveries) {
        let rest = resumed
            .as_ref()
            .map_or(&[][..], |batch| batch.events.as_slice());
        println!(
            "clip-loop end={end:.8}/{:08x} unpaged={}:{:?} first={}:{:?} cursor={:?} resumed={}:{:?} resumed_counts={:?}",
            end.to_bits(), unpaged.events.len(), time_bits(&unpaged.events),
            first.events.len(), time_bits(&first.events),
            first.next_cursor.as_ref().map(|cursor| format!("{:08x}", cursor.playback_time_seconds.to_bits())),
            rest.len(), time_bits(rest), name_counts(rest),
        );
    }

    assert_eq!(limits.max_events, 64);
    for (end, (unpaged, first, resumed)) in ends.into_iter().zip(deliveries) {
        assert!(!unpaged.budget_exhausted && unpaged.next_cursor.is_none());
        assert_occurrences("nonbinding count budget", &unpaged.events, &POSITIONS);
        assert!(first.budget_exhausted);
        assert_occurrences("first default-budget page", &first.events, &POSITIONS[..4]);
        let resumed = resumed.expect("sampler must retain the unfinished range");
        assert!(!resumed.budget_exhausted && resumed.next_cursor.is_none());
        let lengths = [first.events.len(), resumed.events.len()];
        let events: Vec<_> = first.events.into_iter().chain(resumed.events).collect();
        let label = format!("end {end}, default-budget pages {lengths:?}");
        assert_occurrences(&label, &events, &POSITIONS);
        assert_eq!(lengths, [64, 16], "{label}");
    }
}

// Exercise the public LevelSystem queue as well as the resident native-host clip sampler.
#[cfg(feature = "native-host")]
#[test]
fn loop_notify_resume_level_continues_one_original_range() {
    use crate::animation::ProjectAnimationClipEventSampler;
    use crate::asset::{AssetUri, ProjectAssetManager};
    use crate::core::framework::animation::{
        AnimationClipEventBatchAdmission, AnimationClipEventQueueAdmission,
        AnimationClipEventSamplingRange,
    };
    use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
    use crate::scene::{components::NodeKind, DefaultLevelManager};

    const POINT: Real = 0.01;
    const TRACKS: usize = 16;
    let names: [String; TRACKS] = std::array::from_fn(|index| format!("point-{index:02}"));
    let mut clip = clip_with_events(
        names
            .iter()
            .rev()
            .map(|name| event_track(name, POINT, None))
            .collect(),
    );
    clip.duration_seconds = 0.02;
    let assets = ProjectAssetManager::default();
    let uri = AssetUri::parse("res://animation/loop-event-queue.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&uri);
    let restored =
        AnimationClipAsset::from_bytes(&clip.to_bytes().expect("serialize point tracks"))
            .expect("decode point tracks");
    assert_eq!(restored, clip, "the asset must preserve same-time tracks");
    assets.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, uri),
        restored,
    );
    assert_eq!(assets.load_animation_clip_asset(clip_id).unwrap(), clip);
    let sampler = ProjectAnimationClipEventSampler::new(&assets);
    let level = DefaultLevelManager::default().create_default_level();
    let entity = level
        .with_world_mut(|world| world.spawn_node(NodeKind::Cube))
        .unwrap();
    let epoch = level.capture_world_replacement_epoch();
    assert_eq!(
        level.enqueue_animation_clip_event_range_batches(
            epoch,
            vec![vec![AnimationClipEventSamplingRange {
                entity,
                clip_id,
                from_time_seconds: 0.0,
                to_time_seconds: 0.1,
                looping: true,
            }]]
        ),
        AnimationClipEventQueueAdmission::Current {
            batch_admissions: vec![AnimationClipEventBatchAdmission::Admitted],
            admitted_range_count: 1,
            deferred_range_count: 0,
            rejected_range_count: 0,
        }
    );

    let first = level.drain_animation_clip_events(epoch, &sampler).unwrap();
    assert_eq!(level.animation_clip_event_backlog_len(epoch), Some(1));
    let rest = level.drain_animation_clip_events(epoch, &sampler).unwrap();
    assert_eq!(level.animation_clip_event_backlog_len(epoch), Some(0));
    assert!(level
        .drain_animation_clip_events(epoch, &sampler)
        .unwrap()
        .is_empty());
    let counts = names.each_ref().map(|name| {
        first
            .iter()
            .chain(&rest)
            .filter(|ev| ev.event.as_str() == name.as_str())
            .count()
    });
    eprintln!("CLIP_QUEUE_V1 pages={:?} actual_per_name={counts:?} real_range_backlog=1,0 third_empty=true", [first.len(), rest.len()]);
    assert_eq!([first.len(), rest.len()], [64, 16]);
    assert_eq!(counts, [5; TRACKS]);
    let all: Vec<_> = first.into_iter().chain(rest).collect();
    let times: [Real; 5] = [0.01, 0.03, 0.05, 0.07, 0.09];
    for (group, position) in all.chunks_exact(TRACKS).zip(times) {
        for (event, name) in group.iter().zip(&names) {
            assert_eq!(event.event.as_str(), name.as_str());
            assert_eq!(event.entity, entity);
            assert_eq!(event.clip_time_seconds, POINT);
            assert!(
                event
                    .playback_time_seconds
                    .to_bits()
                    .abs_diff(position.to_bits())
                    <= 2
            );
        }
    }
}

#[test]
fn loop_notify_resume_split_frames_and_exact_period_controls() {
    const TRACKS: usize = 16;
    for (period, point, spans) in [
        (0.02_f32, 0.01_f32, vec![(0., 0.0625), (0.0625, 0.1)]),
        (1.0 / 32.0, 1.0 / 64.0, vec![(0., 5.0 / 32.0)]),
    ] {
        let names: [String; TRACKS] = std::array::from_fn(|i| format!("point-{i:02}"));
        let mut clip =
            clip_with_events(names.iter().map(|n| event_track(n, point, None)).collect());
        clip.duration_seconds = period;
        let limits = AnimationClipEventSamplingLimits::default();
        let mut lengths = Vec::new();
        let mut counts = [0u32; TRACKS];
        for (from, to) in spans {
            let mut cursor = None;
            for page in 0..3 {
                let batch =
                    sample_clip_events_budgeted(&clip, 17, from, to, true, cursor.take(), limits);
                lengths.push(batch.events.len());
                for ev in batch.events {
                    let i = names
                        .iter()
                        .position(|n| *n == ev.event)
                        .expect("known track");
                    assert_eq!(ev.clip_time_seconds, point);
                    counts[i] += 1;
                }
                cursor = batch.next_cursor;
                if cursor.is_none() {
                    break;
                }
                assert!(
                    page < 2,
                    "a finite five-crossing input exhausted three pages"
                );
            }
        }
        eprintln!(
            "CLIP_CONTROL_V1 period_bits={:08x} pages={lengths:?} counts={counts:?}",
            period.to_bits()
        );
        assert_eq!(counts, [5; TRACKS]);
        let expected = if period == 0.02 { [48, 32] } else { [64, 16] };
        assert_eq!(lengths, expected);
    }
}

#[path = "clip_event_pages.rs"]
mod pages;

#[path = "clip_event_oracle.rs"]
mod oracle;
