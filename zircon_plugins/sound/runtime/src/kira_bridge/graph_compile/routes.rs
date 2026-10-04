//! 将多级后效果发送展开为每个源轨的下游路由，缓存共享子图以避免重复遍历。
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
) -> Result<(), SoundError> {
    if cache.contains_key(&track) {
        return Ok(());
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
        let target = tracks
            .get(&send.target)
            .copied()
            .ok_or(SoundError::UnknownTrack { track: send.target })?;
        let downstream_input_gain = send.gain * local_track_gain(target);
        expand_track_sends(tracks, send.target, cache, visiting)?;
        let downstream_routes = cache
            .get(&send.target)
            .expect("successful expansion must populate the route cache");
        gains.reserve(1 + downstream_routes.len());
        *gains.entry(send.target).or_default() += send.gain;
        for downstream in downstream_routes {
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
    cache.insert(track, routes);
    Ok(())
}

fn local_track_gain(track: &SoundTrackDescriptor) -> f32 {
    if track.controls.mute {
        0.0
    } else {
        track.controls.gain
    }
}

#[cfg(test)]
#[path = "tests/routes.rs"]
mod tests;
