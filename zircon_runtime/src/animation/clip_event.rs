use crate::asset::ProjectAssetManager;
use crate::core::framework::animation::{
    AnimationClipAsset, AnimationClipEvent, AnimationClipEventSampler,
    AnimationClipEventSamplingBatch, AnimationClipEventSamplingCursor,
    AnimationClipEventSamplingLimits, AnimationClipEventSamplingRequest, AnimationEventTrackAsset,
};
use crate::core::math::Real;
use crate::scene::EntityId;
use std::collections::BinaryHeap;

pub struct ProjectAnimationClipEventSampler<'a> {
    asset_manager: &'a ProjectAssetManager,
}

impl<'a> ProjectAnimationClipEventSampler<'a> {
    pub fn new(asset_manager: &'a ProjectAssetManager) -> Self {
        Self { asset_manager }
    }
}

impl AnimationClipEventSampler for ProjectAnimationClipEventSampler<'_> {
    fn sample_clip_events(
        &self,
        request: AnimationClipEventSamplingRequest,
    ) -> Option<AnimationClipEventSamplingBatch> {
        let clip = self
            .asset_manager
            .load_animation_clip_asset(request.clip_id)
            .ok()?;
        Some(sample_clip_events_budgeted(
            &clip,
            request.entity,
            request.from_time_seconds,
            request.to_time_seconds,
            request.looping,
            Some(request.cursor),
            request.limits,
        ))
    }
}

#[derive(Clone, Copy, Debug)]
struct EventCandidate<'track> {
    track_index: usize,
    playback_time_seconds: Real,
    event: &'track str,
}

impl PartialEq for EventCandidate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.playback_time_seconds
            .total_cmp(&other.playback_time_seconds)
            .is_eq()
            && self.event == other.event
            && self.track_index == other.track_index
    }
}

impl Eq for EventCandidate<'_> {}

impl PartialOrd for EventCandidate<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventCandidate<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        record_candidate_comparison();
        other
            .playback_time_seconds
            .total_cmp(&self.playback_time_seconds)
            .then_with(|| other.event.cmp(self.event))
            .then_with(|| other.track_index.cmp(&self.track_index))
    }
}

/// Samples one bounded, resumable portion of a clip-event range.
///
/// A batch never drops an event because of its budget. If a single event exceeds the byte
/// limit, it is emitted by itself and reported as oversized so the cursor can keep moving.
fn sample_clip_events_budgeted(
    clip: &AnimationClipAsset,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    looping: bool,
    cursor: Option<AnimationClipEventSamplingCursor>,
    limits: AnimationClipEventSamplingLimits,
) -> AnimationClipEventSamplingBatch {
    if clip.event_tracks.is_empty()
        || !from_time_seconds.is_finite()
        || !to_time_seconds.is_finite()
        || to_time_seconds <= from_time_seconds
        || !limits.max_playback_span_seconds.is_finite()
        || limits.max_playback_span_seconds <= Real::EPSILON
        || limits.max_events == 0
    {
        return AnimationClipEventSamplingBatch::default();
    }

    let Some((range_start, range_end, duration_seconds)) =
        event_sampling_range(clip, from_time_seconds, to_time_seconds, looping)
    else {
        return AnimationClipEventSamplingBatch::default();
    };
    let cursor =
        cursor.unwrap_or_else(|| AnimationClipEventSamplingCursor::at_range_start(range_start));
    let range_cursor = cursor.playback_time_seconds.clamp(range_start, range_end);
    let cursor = AnimationClipEventSamplingCursor {
        playback_time_seconds: range_cursor,
        ..cursor
    };
    if range_cursor >= range_end && cursor.last_event.is_none() {
        return AnimationClipEventSamplingBatch::default();
    }

    let batch_end = (range_cursor + limits.max_playback_span_seconds).min(range_end);
    let mut candidates = clip
        .event_tracks
        .iter()
        .enumerate()
        .filter_map(|(track_index, track)| {
            event_candidate(
                track,
                track_index,
                duration_seconds,
                looping,
                &cursor,
                batch_end,
            )
        })
        .collect::<BinaryHeap<_>>();
    let mut batch = AnimationClipEventSamplingBatch {
        playback_span_seconds: batch_end - range_cursor,
        ..AnimationClipEventSamplingBatch::default()
    };
    let mut last_cursor = cursor.clone();

    while batch.events.len() < limits.max_events {
        let Some(candidate) = candidates.peek().copied() else {
            break;
        };
        if candidate.playback_time_seconds > batch_end {
            break;
        }

        let track = &clip.event_tracks[candidate.track_index];
        let event_bytes = event_text_bytes(track);
        if !batch.events.is_empty()
            && batch.emitted_event_bytes.saturating_add(event_bytes) > limits.max_event_bytes
        {
            batch.budget_exhausted = true;
            break;
        }
        if batch.events.is_empty() && event_bytes > limits.max_event_bytes {
            batch.oversized_event_count = 1;
            batch.budget_exhausted = true;
        }
        candidates.pop();

        batch.emitted_event_bytes = batch.emitted_event_bytes.saturating_add(event_bytes);
        batch.events.push(AnimationClipEvent {
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

        if looping {
            candidates.push(EventCandidate {
                playback_time_seconds: candidate.playback_time_seconds + duration_seconds,
                ..candidate
            });
        }
    }

    let candidates_remain = candidates
        .peek()
        .is_some_and(|candidate| candidate.playback_time_seconds <= batch_end);
    if candidates_remain {
        batch.budget_exhausted = true;
        batch.next_cursor = Some(last_cursor);
    } else if batch_end < range_end {
        batch.next_cursor = Some(AnimationClipEventSamplingCursor::at_range_start(batch_end));
    }
    batch
}

fn event_sampling_range(
    clip: &AnimationClipAsset,
    from_time_seconds: Real,
    to_time_seconds: Real,
    looping: bool,
) -> Option<(Real, Real, Real)> {
    if looping {
        let duration_seconds = finite_positive_duration(clip.duration_seconds)?;
        let range_start = from_time_seconds.max(0.0);
        let range_end = to_time_seconds.max(0.0);
        (range_end > range_start).then_some((range_start, range_end, duration_seconds))
    } else {
        let duration_seconds = finite_positive_duration(clip.duration_seconds);
        let range_start = duration_seconds
            .map(|duration| from_time_seconds.min(duration))
            .unwrap_or(from_time_seconds)
            .max(0.0);
        let range_end = duration_seconds
            .map(|duration| to_time_seconds.min(duration))
            .unwrap_or(to_time_seconds)
            .max(0.0);
        (range_end > range_start).then_some((range_start, range_end, 0.0))
    }
}

fn event_candidate<'track>(
    track: &'track AnimationEventTrackAsset,
    track_index: usize,
    duration_seconds: Real,
    looping: bool,
    cursor: &AnimationClipEventSamplingCursor,
    batch_end: Real,
) -> Option<EventCandidate<'track>> {
    if !track.time_seconds.is_finite() || track.time_seconds < 0.0 {
        return None;
    }
    let playback_time_seconds = if looping {
        if track.time_seconds > duration_seconds {
            return None;
        }
        let occurrence = first_looping_occurrence_at_or_after(
            track.time_seconds,
            duration_seconds,
            cursor.playback_time_seconds,
        );
        if occurrence.total_cmp(&cursor.playback_time_seconds).is_eq()
            && !event_is_after_cursor(occurrence, track, track_index, cursor)
        {
            occurrence + duration_seconds
        } else {
            occurrence
        }
    } else {
        track.time_seconds
    };
    if playback_time_seconds > batch_end
        || !event_is_after_cursor(playback_time_seconds, track, track_index, cursor)
    {
        return None;
    }
    Some(EventCandidate {
        track_index,
        playback_time_seconds,
        event: &track.event,
    })
}

fn first_looping_occurrence_at_or_after(
    clip_time_seconds: Real,
    duration_seconds: Real,
    playback_time_seconds: Real,
) -> Real {
    if playback_time_seconds <= clip_time_seconds {
        return clip_time_seconds;
    }
    let loop_index = ((playback_time_seconds - clip_time_seconds) / duration_seconds)
        .ceil()
        .max(0.0);
    let occurrence = clip_time_seconds + loop_index * duration_seconds;
    if occurrence < playback_time_seconds {
        occurrence + duration_seconds
    } else {
        occurrence
    }
}

fn event_is_after_cursor(
    playback_time_seconds: Real,
    track: &AnimationEventTrackAsset,
    track_index: usize,
    cursor: &AnimationClipEventSamplingCursor,
) -> bool {
    match playback_time_seconds.total_cmp(&cursor.playback_time_seconds) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => cursor.last_event.as_ref().is_some_and(|last_event| {
            track
                .event
                .as_str()
                .cmp(last_event.as_ref())
                .then_with(|| track_index.cmp(&cursor.last_track_index))
                .is_gt()
        }),
    }
}

#[cfg(all(test, debug_assertions))]
thread_local! {
    static CANDIDATE_COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(all(test, debug_assertions))]
fn record_candidate_comparison() {
    CANDIDATE_COMPARISONS.with(|comparisons| comparisons.set(comparisons.get() + 1));
}

#[cfg(not(all(test, debug_assertions)))]
fn record_candidate_comparison() {}

#[cfg(all(test, debug_assertions))]
fn take_candidate_comparisons() -> usize {
    CANDIDATE_COMPARISONS.with(|comparisons| comparisons.replace(0))
}

#[cfg(all(test, not(debug_assertions)))]
fn take_candidate_comparisons() -> usize {
    0
}

fn event_text_bytes(track: &AnimationEventTrackAsset) -> usize {
    track.event.len()
        + track.target_id.as_ref().map_or(0, String::len)
        + track.payload.as_ref().map_or(0, String::len)
}

fn finite_positive_duration(duration_seconds: Real) -> Option<Real> {
    (duration_seconds.is_finite() && duration_seconds > Real::EPSILON).then_some(duration_seconds)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::animation::{
        AnimationClipAsset, AnimationClipEvent, AnimationClipEventSamplingCursor,
        AnimationEventTrackAsset,
    };
    use crate::core::math::Real;
    use crate::core::resource::{AssetReference, ResourceLocator};

    use super::{
        AnimationClipEventSamplingLimits, EventCandidate, event_candidate,
        sample_clip_events_budgeted, take_candidate_comparisons,
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

        let second =
            sample_clip_events_budgeted(&clip, 8, 0.0, 1.0, false, first.next_cursor, limits);
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

        let second =
            sample_clip_events_budgeted(&clip, 9, 0.0, 1.0, false, first.next_cursor, limits);
        assert_eq!(second.events[0].event, "beta");
        assert!(second.next_cursor.is_none());
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

        let second =
            sample_clip_events_budgeted(&clip, 10, 0.0, 2.0, true, first.next_cursor, limits);
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

        let second =
            sample_clip_events_budgeted(&clip, 12, 0.0, 1.0, false, first.next_cursor, limits);
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
                let sampled = sample_clip_events_budgeted(
                    black_box(&clip),
                    13,
                    0.0,
                    1.0,
                    false,
                    None,
                    limits,
                );
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
                event_candidate(track, track_index, 0.0, false, &cursor, 1.0)
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

    fn event_track(
        event: &str,
        time_seconds: Real,
        payload: Option<&str>,
    ) -> AnimationEventTrackAsset {
        AnimationEventTrackAsset {
            target_id: None,
            event: event.to_string(),
            time_seconds,
            payload: payload.map(str::to_string),
        }
    }
}
