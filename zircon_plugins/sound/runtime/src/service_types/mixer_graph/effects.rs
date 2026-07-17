use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundTrackId,
};

use super::super::DefaultSoundManager;
use super::sync::mutate_graph;
use crate::kira_bridge::validate_effect;

impl DefaultSoundManager {
    pub(in crate::service_types) fn add_or_update_effect_impl(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError> {
        validate_effect(&effect)?;
        mutate_graph(
            self,
            |graph| {
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
                    *existing = effect.clone();
                } else {
                    graph_track.effects.push(effect.clone());
                }
                Ok(())
            },
            |_| {},
        )
    }

    pub(in crate::service_types) fn remove_effect_impl(
        &self,
        track: SoundTrackId,
        effect: SoundEffectId,
    ) -> Result<(), SoundError> {
        mutate_graph(
            self,
            |graph| {
                let graph_track = graph
                    .tracks
                    .iter_mut()
                    .find(|candidate| candidate.id == track)
                    .ok_or(SoundError::UnknownTrack { track })?;
                let before = graph_track.effects.len();
                graph_track
                    .effects
                    .retain(|candidate| candidate.id != effect);
                if before == graph_track.effects.len() {
                    return Err(SoundError::UnknownEffect { effect });
                }
                Ok(())
            },
            |_| {},
        )
    }
}
