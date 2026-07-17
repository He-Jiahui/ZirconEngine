use zircon_runtime::core::framework::sound::{SoundError, SoundTrackId, SoundTrackSend};

use super::super::DefaultSoundManager;
use super::sync::mutate_graph;

impl DefaultSoundManager {
    pub(in crate::service_types) fn add_or_update_track_send_impl(
        &self,
        track: SoundTrackId,
        send: SoundTrackSend,
    ) -> Result<(), SoundError> {
        if !send.gain.is_finite() {
            return Err(SoundError::InvalidParameter(
                "track send gain must be finite".to_string(),
            ));
        }
        mutate_graph(
            self,
            |graph| {
                let track_index = graph
                    .tracks
                    .iter()
                    .position(|candidate| candidate.id == track)
                    .ok_or(SoundError::UnknownTrack { track })?;
                if !graph
                    .tracks
                    .iter()
                    .any(|candidate| candidate.id == send.target)
                {
                    return Err(SoundError::UnknownTrack { track: send.target });
                }
                if let Some(existing) = graph.tracks[track_index]
                    .sends
                    .iter_mut()
                    .find(|candidate| candidate.target == send.target)
                {
                    *existing = send.clone();
                } else {
                    graph.tracks[track_index].sends.push(send.clone());
                }
                Ok(())
            },
            |_| {},
        )
    }

    pub(in crate::service_types) fn remove_track_send_impl(
        &self,
        track: SoundTrackId,
        target: SoundTrackId,
    ) -> Result<(), SoundError> {
        mutate_graph(
            self,
            |graph| {
                let graph_track = graph
                    .tracks
                    .iter_mut()
                    .find(|candidate| candidate.id == track)
                    .ok_or(SoundError::UnknownTrack { track })?;
                let before = graph_track.sends.len();
                graph_track
                    .sends
                    .retain(|candidate| candidate.target != target);
                if before == graph_track.sends.len() {
                    return Err(SoundError::UnknownSend { track, target });
                }
                Ok(())
            },
            |_| {},
        )
    }
}
