use zircon_runtime::core::framework::sound::{SoundError, SoundTrackId, SoundTrackSend};

use crate::engine::validation::validate_graph;

use super::super::DefaultSoundManager;

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
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let mut graph = state.graph.clone();
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
            *existing = send;
        } else {
            graph.tracks[track_index].sends.push(send);
        }
        validate_graph(&graph)?;
        state.graph = graph;
        Ok(())
    }

    pub(in crate::service_types) fn remove_track_send_impl(
        &self,
        track: SoundTrackId,
        target: SoundTrackId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let graph_track = state
            .graph
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
    }
}
