use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{SoundMixerGraph, SoundTrackId};

pub(in crate::engine::render) fn solo_tracks(graph: &SoundMixerGraph) -> HashSet<SoundTrackId> {
    graph
        .tracks
        .iter()
        .filter(|track| track.controls.solo)
        .map(|track| track.id)
        .collect()
}

pub(in crate::engine::render) fn accepts_direct_input(
    track: SoundTrackId,
    solo_tracks: &HashSet<SoundTrackId>,
) -> bool {
    solo_tracks.is_empty() || solo_tracks.contains(&track)
}
