use zircon_runtime::core::framework::sound::{SoundError, SoundTrackDescriptor, SoundTrackId};

use super::super::DefaultSoundManager;
use super::sync::mutate_graph;

impl DefaultSoundManager {
    pub(in crate::service_types) fn add_or_update_track_impl(
        &self,
        track: SoundTrackDescriptor,
    ) -> Result<(), SoundError> {
        mutate_graph(
            self,
            |graph| {
                if let Some(existing) = graph
                    .tracks
                    .iter_mut()
                    .find(|candidate| candidate.id == track.id)
                {
                    *existing = track.clone();
                } else {
                    graph.tracks.push(track.clone());
                }
                Ok(())
            },
            |_| {},
        )
    }

    pub(in crate::service_types) fn remove_track_impl(
        &self,
        track: SoundTrackId,
    ) -> Result<(), SoundError> {
        if track == SoundTrackId::master() {
            return Err(SoundError::InvalidMixerGraph(
                "master track cannot be removed".to_string(),
            ));
        }
        mutate_graph(
            self,
            |graph| {
                let before = graph.tracks.len();
                graph.tracks.retain(|candidate| candidate.id != track);
                if before == graph.tracks.len() {
                    return Err(SoundError::UnknownTrack { track });
                }
                Ok(())
            },
            |state| {
                for playback in state.playbacks.values_mut() {
                    if playback.output_track == track {
                        playback.output_track = SoundTrackId::master();
                    }
                }
                for source in state.sources.values_mut() {
                    if source.descriptor.output_track == track {
                        source.descriptor.output_track = SoundTrackId::master();
                    }
                }
            },
        )
    }
}
