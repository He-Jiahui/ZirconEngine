use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundTrackId,
};

use crate::engine::validation::{validate_effect, validate_graph};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn add_or_update_effect_impl(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError> {
        validate_effect(&effect)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let mut graph = state.graph.clone();
        let graph_track = graph
            .tracks
            .iter_mut()
            .find(|candidate| candidate.id == track)
            .ok_or(SoundError::UnknownTrack { track })?;
        if let Some(existing) = graph_track
            .effects
            .iter_mut()
            .find(|candidate| candidate.id == effect.id)
        {
            *existing = effect;
        } else {
            graph_track.effects.push(effect);
        }
        validate_graph(&graph)?;
        state.graph = graph;
        Ok(())
    }

    pub(in crate::service_types) fn remove_effect_impl(
        &self,
        track: SoundTrackId,
        effect: SoundEffectId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let track = state
            .graph
            .tracks
            .iter_mut()
            .find(|candidate| candidate.id == track)
            .ok_or(SoundError::UnknownTrack { track })?;
        let before = track.effects.len();
        track.effects.retain(|candidate| candidate.id != effect);
        if before == track.effects.len() {
            return Err(SoundError::UnknownEffect { effect });
        }
        Ok(())
    }
}
