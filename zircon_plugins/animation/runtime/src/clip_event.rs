use zircon_runtime::asset::AnimationClipAsset;
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::EntityId;

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

pub fn sample_clip_events(
    clip: &AnimationClipAsset,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    looping: bool,
) -> Vec<AnimationClipEvent> {
    if clip.event_tracks.is_empty()
        || !from_time_seconds.is_finite()
        || !to_time_seconds.is_finite()
        || to_time_seconds <= from_time_seconds
    {
        return Vec::new();
    }

    if looping {
        return sample_looping_clip_events(clip, entity, from_time_seconds, to_time_seconds);
    }

    let duration_seconds = finite_positive_duration(clip.duration_seconds);
    let end_time = duration_seconds
        .map(|duration| to_time_seconds.min(duration))
        .unwrap_or(to_time_seconds)
        .max(0.0);
    let start_time = duration_seconds
        .map(|duration| from_time_seconds.min(duration))
        .unwrap_or(from_time_seconds)
        .max(0.0);
    if end_time <= start_time {
        return Vec::new();
    }

    let mut events = clip
        .event_tracks
        .iter()
        .filter(|track| {
            track.time_seconds.is_finite()
                && track.time_seconds > start_time
                && track.time_seconds <= end_time
        })
        .map(|track| AnimationClipEvent {
            entity,
            target_id: track.target_id.clone(),
            event: track.event.clone(),
            payload: track.payload.clone(),
            clip_time_seconds: track.time_seconds,
            playback_time_seconds: track.time_seconds,
        })
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.playback_time_seconds
            .total_cmp(&right.playback_time_seconds)
            .then_with(|| left.event.cmp(&right.event))
    });
    events
}

fn sample_looping_clip_events(
    clip: &AnimationClipAsset,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<AnimationClipEvent> {
    let Some(duration_seconds) = finite_positive_duration(clip.duration_seconds) else {
        return Vec::new();
    };
    let start_time = from_time_seconds.max(0.0);
    let end_time = to_time_seconds.max(0.0);
    if end_time <= start_time {
        return Vec::new();
    }

    let mut events = Vec::new();
    for track in &clip.event_tracks {
        if !track.time_seconds.is_finite()
            || track.time_seconds < 0.0
            || track.time_seconds > duration_seconds
        {
            continue;
        }

        let mut occurrence =
            first_looping_occurrence_after(track.time_seconds, duration_seconds, start_time);
        while occurrence <= end_time {
            events.push(AnimationClipEvent {
                entity,
                target_id: track.target_id.clone(),
                event: track.event.clone(),
                payload: track.payload.clone(),
                clip_time_seconds: track.time_seconds,
                playback_time_seconds: occurrence,
            });
            occurrence += duration_seconds;
        }
    }
    events.sort_by(|left, right| {
        left.playback_time_seconds
            .total_cmp(&right.playback_time_seconds)
            .then_with(|| left.event.cmp(&right.event))
    });
    events
}

fn first_looping_occurrence_after(
    clip_time_seconds: Real,
    duration_seconds: Real,
    after_time_seconds: Real,
) -> Real {
    let loops_before = ((after_time_seconds - clip_time_seconds) / duration_seconds).floor();
    let loop_index = (loops_before as i64 + 1).max(0) as Real;
    clip_time_seconds + loop_index * duration_seconds
}

fn finite_positive_duration(duration_seconds: Real) -> Option<Real> {
    (duration_seconds.is_finite() && duration_seconds > Real::EPSILON).then_some(duration_seconds)
}
