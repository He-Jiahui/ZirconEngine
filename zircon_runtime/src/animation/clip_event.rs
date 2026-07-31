use crate::core::framework::animation::{AnimationClipAsset, AnimationEventTrackAsset};
use crate::core::math::Real;
use crate::scene::EntityId;

/// Runtime event emitted when an animation clip playback range crosses an event track.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClipEvent {
    pub entity: EntityId,
    pub target_id: Option<String>,
    pub event: String,
    pub payload: Option<String>,
    pub clip_time_seconds: Real,
    pub playback_time_seconds: Real,
}

/// A bounded portion of playback time considered by one event drain.
///
/// The cursor is retained by the Level-owned pending-event queue so a large seek never expands
/// all looping occurrences in one frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnimationClipEventSamplingCursor {
    playback_time_seconds: Real,
    last_event: Option<Box<str>>,
    last_track_index: usize,
}

impl AnimationClipEventSamplingCursor {
    pub(crate) fn at_range_start(playback_time_seconds: Real) -> Self {
        Self {
            playback_time_seconds,
            last_event: None,
            last_track_index: 0,
        }
    }
}

/// Per-frame bounds for draining clip events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnimationClipEventSamplingLimits {
    pub(crate) max_events: usize,
    pub(crate) max_event_bytes: usize,
    pub(crate) max_playback_span_seconds: Real,
}

impl Default for AnimationClipEventSamplingLimits {
    fn default() -> Self {
        Self {
            max_events: 64,
            max_event_bytes: 64 * 1024,
            max_playback_span_seconds: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AnimationClipEventSamplingBatch {
    pub(crate) events: Vec<AnimationClipEvent>,
    pub(crate) next_cursor: Option<AnimationClipEventSamplingCursor>,
    pub(crate) emitted_event_bytes: usize,
    pub(crate) playback_span_seconds: Real,
    pub(crate) budget_exhausted: bool,
    pub(crate) oversized_event_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct EventCandidate {
    track_index: usize,
    playback_time_seconds: Real,
}

/// Samples one bounded, resumable portion of a clip-event range.
///
/// A batch never drops an event because of its budget. If a single event exceeds the byte
/// limit, it is emitted by itself and reported as oversized so the cursor can keep moving.
pub(crate) fn sample_clip_events_budgeted(
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
        .collect::<Vec<_>>();
    let mut batch = AnimationClipEventSamplingBatch {
        playback_span_seconds: batch_end - range_cursor,
        ..AnimationClipEventSamplingBatch::default()
    };
    let mut last_cursor = cursor.clone();

    while batch.events.len() < limits.max_events {
        let Some((candidate_index, candidate)) = candidates
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| compare_candidates(&clip.event_tracks, left, right))
            .map(|(index, candidate)| (index, *candidate))
        else {
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
            candidates[candidate_index].playback_time_seconds += duration_seconds
        } else {
            candidates.remove(candidate_index);
        }
    }

    let candidates_remain = candidates
        .iter()
        .any(|candidate| candidate.playback_time_seconds <= batch_end);
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

fn event_candidate(
    track: &AnimationEventTrackAsset,
    track_index: usize,
    duration_seconds: Real,
    looping: bool,
    cursor: &AnimationClipEventSamplingCursor,
    batch_end: Real,
) -> Option<EventCandidate> {
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

fn compare_candidates(
    tracks: &[AnimationEventTrackAsset],
    left: &EventCandidate,
    right: &EventCandidate,
) -> std::cmp::Ordering {
    left.playback_time_seconds
        .total_cmp(&right.playback_time_seconds)
        .then_with(|| {
            tracks[left.track_index]
                .event
                .cmp(&tracks[right.track_index].event)
        })
        .then_with(|| left.track_index.cmp(&right.track_index))
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
    use crate::core::framework::animation::{AnimationClipAsset, AnimationEventTrackAsset};
    use crate::core::math::Real;
    use crate::core::resource::{AssetReference, ResourceLocator};

    use super::{sample_clip_events_budgeted, AnimationClipEventSamplingLimits};

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
