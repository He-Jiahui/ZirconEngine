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
    reverse: bool,
}

impl PartialEq for EventCandidate<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.playback_time_seconds
            .total_cmp(&other.playback_time_seconds)
            .is_eq()
            && self.event == other.event
            && self.track_index == other.track_index
            && self.reverse == other.reverse
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
        let self_priority = if self.reverse {
            self.playback_time_seconds
        } else {
            -self.playback_time_seconds
        };
        let other_priority = if other.reverse {
            other.playback_time_seconds
        } else {
            -other.playback_time_seconds
        };
        self_priority
            .total_cmp(&other_priority)
            .then_with(|| other.event.cmp(self.event))
            .then_with(|| other.track_index.cmp(&self.track_index))
            .then_with(|| self.reverse.cmp(&other.reverse))
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
        || to_time_seconds == from_time_seconds
        || !limits.max_playback_span_seconds.is_finite()
        || limits.max_playback_span_seconds <= Real::EPSILON
        || limits.max_events == 0
    {
        return AnimationClipEventSamplingBatch::default();
    }

    let Some((range_from, range_to, duration_seconds)) =
        event_sampling_range(clip, from_time_seconds, to_time_seconds, looping)
    else {
        return AnimationClipEventSamplingBatch::default();
    };
    let cursor =
        cursor.unwrap_or_else(|| AnimationClipEventSamplingCursor::at_range_start(range_from));
    let reverse = range_to < range_from;
    let range_cursor = cursor
        .playback_time_seconds
        .clamp(range_from.min(range_to), range_from.max(range_to));
    let cursor = AnimationClipEventSamplingCursor {
        playback_time_seconds: range_cursor,
        ..cursor
    };
    if range_cursor == range_to && cursor.last_event.is_none() {
        return AnimationClipEventSamplingBatch::default();
    }

    let batch_end = if reverse {
        (range_cursor - limits.max_playback_span_seconds).max(range_to)
    } else {
        (range_cursor + limits.max_playback_span_seconds).min(range_to)
    };
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
                reverse,
                &cursor,
                batch_end,
            )
        })
        .collect::<BinaryHeap<_>>();
    let mut batch = AnimationClipEventSamplingBatch {
        events: Vec::with_capacity(limits.max_events),
        playback_span_seconds: (batch_end - range_cursor).abs(),
        ..AnimationClipEventSamplingBatch::default()
    };
    let mut last_cursor = cursor.clone();

    while batch.events.len() < limits.max_events {
        let Some(candidate) = candidates.peek().copied() else {
            break;
        };
        if !playback_time_is_within_batch(
            candidate.playback_time_seconds,
            range_cursor,
            batch_end,
            reverse,
        ) {
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
            let next_playback_time = if reverse {
                candidate.playback_time_seconds - duration_seconds
            } else {
                candidate.playback_time_seconds + duration_seconds
            };
            let advances = if reverse {
                next_playback_time < candidate.playback_time_seconds
            } else {
                next_playback_time > candidate.playback_time_seconds
            };
            if next_playback_time.is_finite() && advances {
                candidates.push(EventCandidate {
                    playback_time_seconds: next_playback_time,
                    ..candidate
                });
            }
        }
    }

    let candidates_remain = candidates.peek().is_some_and(|candidate| {
        playback_time_is_within_batch(
            candidate.playback_time_seconds,
            range_cursor,
            batch_end,
            reverse,
        )
    });
    if candidates_remain {
        batch.budget_exhausted = true;
        batch.next_cursor = Some(last_cursor);
    } else if batch_end != range_to {
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
        let range_from = from_time_seconds.max(0.0);
        let range_to = to_time_seconds.max(0.0);
        (range_from != range_to).then_some((range_from, range_to, duration_seconds))
    } else {
        let duration_seconds = finite_positive_duration(clip.duration_seconds);
        let range_from = duration_seconds
            .map(|duration| from_time_seconds.min(duration))
            .unwrap_or(from_time_seconds)
            .max(0.0);
        let range_to = duration_seconds
            .map(|duration| to_time_seconds.min(duration))
            .unwrap_or(to_time_seconds)
            .max(0.0);
        (range_from != range_to).then_some((range_from, range_to, 0.0))
    }
}

fn event_candidate<'track>(
    track: &'track AnimationEventTrackAsset,
    track_index: usize,
    duration_seconds: Real,
    looping: bool,
    reverse: bool,
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
        // Match the side of a loop seam used by animation playback: the end event belongs
        // to forward traversal and the start event belongs to reverse traversal.
        if (reverse && track.time_seconds == duration_seconds)
            || (!reverse && track.time_seconds == 0.0)
        {
            return None;
        }
        let occurrence = if reverse {
            last_looping_occurrence_at_or_before(
                track.time_seconds,
                duration_seconds,
                cursor.playback_time_seconds,
            )
        } else {
            first_looping_occurrence_at_or_after(
                track.time_seconds,
                duration_seconds,
                cursor.playback_time_seconds,
            )
        };
        if occurrence.total_cmp(&cursor.playback_time_seconds).is_eq()
            && !event_is_after_cursor(occurrence, track, track_index, cursor, reverse)
        {
            if reverse {
                occurrence - duration_seconds
            } else {
                occurrence + duration_seconds
            }
        } else {
            occurrence
        }
    } else {
        track.time_seconds
    };
    if !playback_time_seconds.is_finite()
        || !playback_time_is_within_batch(
            playback_time_seconds,
            cursor.playback_time_seconds,
            batch_end,
            reverse,
        )
        || !event_is_after_cursor(playback_time_seconds, track, track_index, cursor, reverse)
    {
        return None;
    }
    Some(EventCandidate {
        track_index,
        playback_time_seconds,
        event: &track.event,
        reverse,
    })
}

fn playback_time_is_within_batch(
    playback_time_seconds: Real,
    range_cursor: Real,
    batch_end: Real,
    reverse: bool,
) -> bool {
    if reverse {
        playback_time_seconds <= range_cursor && playback_time_seconds >= batch_end
    } else {
        playback_time_seconds >= range_cursor && playback_time_seconds <= batch_end
    }
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

fn last_looping_occurrence_at_or_before(
    clip_time_seconds: Real,
    duration_seconds: Real,
    playback_time_seconds: Real,
) -> Real {
    let loop_index = ((playback_time_seconds - clip_time_seconds) / duration_seconds).floor();
    let occurrence = clip_time_seconds + loop_index * duration_seconds;
    if occurrence > playback_time_seconds {
        occurrence - duration_seconds
    } else {
        occurrence
    }
}

fn event_is_after_cursor(
    playback_time_seconds: Real,
    track: &AnimationEventTrackAsset,
    track_index: usize,
    cursor: &AnimationClipEventSamplingCursor,
    reverse: bool,
) -> bool {
    match playback_time_seconds.total_cmp(&cursor.playback_time_seconds) {
        std::cmp::Ordering::Greater => !reverse,
        std::cmp::Ordering::Less => reverse,
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
#[path = "tests/clip_event.rs"]
mod tests;
