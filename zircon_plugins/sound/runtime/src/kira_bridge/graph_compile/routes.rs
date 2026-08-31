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
mod tests {
    use super::*;

    #[test]
    fn shared_downstream_routes_are_reused_from_cache() {
        let a = SoundTrackId::new(1);
        let b = SoundTrackId::new(2);
        let c = SoundTrackId::new(3);
        let d = SoundTrackId::new(4);
        let mut tracks = [
            track(a, &[(b, 1.0), (c, 2.0)]),
            track(b, &[(d, 0.5)]),
            track(c, &[(d, 0.25)]),
            track(d, &[]),
        ];
        for track in &mut tracks {
            track.parent = None;
        }
        let lookup = tracks.iter().map(|track| (track.id, track)).collect();

        let expanded = expanded_post_effect_sends(&lookup).unwrap();

        assert_eq!(
            expanded.get(&a).map(Vec::as_slice),
            Some([send(b, 1.0), send(c, 2.0), send(d, 1.0),].as_slice())
        );
        assert_eq!(
            expanded.get(&b).map(Vec::as_slice),
            Some([send(d, 0.5)].as_slice())
        );
        assert_eq!(
            expanded.get(&c).map(Vec::as_slice),
            Some([send(d, 0.25)].as_slice())
        );
        assert_eq!(expanded.get(&d).map(Vec::as_slice), Some([].as_slice()));
    }

    fn track(id: SoundTrackId, sends: &[(SoundTrackId, f32)]) -> SoundTrackDescriptor {
        let mut track = SoundTrackDescriptor::child(id, format!("Track {}", id.raw()));
        track.sends = sends
            .iter()
            .map(|(target, gain)| send(*target, *gain))
            .collect();
        track
    }

    fn send(target: SoundTrackId, gain: f32) -> SoundTrackSend {
        SoundTrackSend {
            target,
            gain,
            pre_effects: false,
        }
    }
}
