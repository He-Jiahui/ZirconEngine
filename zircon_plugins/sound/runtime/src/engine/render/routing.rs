use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{SoundMixerGraph, SoundTrackId};

pub(super) fn solo_tracks(graph: &SoundMixerGraph) -> HashSet<SoundTrackId> {
    graph
        .tracks
        .iter()
        .filter(|track| track.controls.solo)
        .map(|track| track.id)
        .collect()
}

pub(super) fn accepts_direct_input(
    track: SoundTrackId,
    solo_tracks: &HashSet<SoundTrackId>,
) -> bool {
    solo_tracks.is_empty() || solo_tracks.contains(&track)
}

pub(super) fn add_scaled(destination: &mut [f32], source: &[f32], gain: f32) {
    for (destination, source) in destination.iter_mut().zip(source.iter().copied()) {
        *destination += source * gain;
    }
}
