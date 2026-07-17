use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{
    SoundError, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

pub(super) fn expanded_post_effect_sends(
    tracks: &HashMap<SoundTrackId, &SoundTrackDescriptor>,
) -> Result<HashMap<SoundTrackId, Vec<SoundTrackSend>>, SoundError> {
    let mut cache = HashMap::new();
    let mut visiting = HashSet::new();
    for track in tracks.keys().copied() {
        expand_track_sends(tracks, track, &mut cache, &mut visiting)?;
    }
    Ok(cache)
}

fn expand_track_sends(
    tracks: &HashMap<SoundTrackId, &SoundTrackDescriptor>,
    track: SoundTrackId,
    cache: &mut HashMap<SoundTrackId, Vec<SoundTrackSend>>,
    visiting: &mut HashSet<SoundTrackId>,
) -> Result<Vec<SoundTrackSend>, SoundError> {
    if let Some(routes) = cache.get(&track) {
        return Ok(routes.clone());
    }
    if !visiting.insert(track) {
        return Err(SoundError::InvalidMixerGraph(
            "track send routing contains a cycle".to_string(),
        ));
    }
    let descriptor = tracks
        .get(&track)
        .copied()
        .ok_or(SoundError::UnknownTrack { track })?;
    let mut gains = HashMap::<SoundTrackId, f32>::new();
    for send in &descriptor.sends {
        *gains.entry(send.target).or_default() += send.gain;
        let target = tracks
            .get(&send.target)
            .copied()
            .ok_or(SoundError::UnknownTrack { track: send.target })?;
        let downstream_input_gain = send.gain * local_track_gain(target);
        for downstream in expand_track_sends(tracks, send.target, cache, visiting)? {
            *gains.entry(downstream.target).or_default() += downstream_input_gain * downstream.gain;
        }
    }
    visiting.remove(&track);
    let mut routes = gains
        .into_iter()
        .map(|(target, gain)| SoundTrackSend {
            target,
            gain,
            pre_effects: false,
        })
        .collect::<Vec<_>>();
    routes.sort_by_key(|send| send.target.raw());
    cache.insert(track, routes.clone());
    Ok(routes)
}

fn local_track_gain(track: &SoundTrackDescriptor) -> f32 {
    if track.controls.mute {
        0.0
    } else {
        track.controls.gain
    }
}
