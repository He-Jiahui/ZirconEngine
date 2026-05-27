use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundMixerGraph, SoundMixerPresetDescriptor,
    SoundMixerSnapshot, SoundTrackDescriptor, SoundTrackId, SoundTrackMeter, SoundTrackSend,
};

use crate::engine::validation::{validate_effect, validate_graph};
use crate::mixer_configuration::configure_mixer_graph;
use crate::presets::built_in_mixer_presets;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn available_mixer_presets_impl(
        &self,
    ) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError> {
        Ok(built_in_mixer_presets(&self.config()))
    }

    pub(super) fn apply_mixer_preset_impl(&self, locator: &str) -> Result<(), SoundError> {
        let config = self.config();
        let preset = built_in_mixer_presets(&config)
            .into_iter()
            .find(|preset| preset.locator == locator)
            .ok_or_else(|| SoundError::InvalidLocator {
                locator: locator.to_string(),
            })?;
        validate_graph(&preset.graph)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state.graph = preset.graph;
        state.effect_states.clear();
        state.track_states.clear();
        state.hrtf_states.clear();
        state.meters = vec![SoundTrackMeter::silent(SoundTrackId::master())];
        let track_ids = state
            .graph
            .tracks
            .iter()
            .map(|track| track.id)
            .collect::<HashSet<_>>();
        for playback in state.playbacks.values_mut() {
            if !track_ids.contains(&playback.output_track) {
                playback.output_track = SoundTrackId::master();
            }
        }
        for source in state.sources.values_mut() {
            if !track_ids.contains(&source.descriptor.output_track) {
                source.descriptor.output_track = SoundTrackId::master();
            }
            source
                .descriptor
                .sends
                .retain(|send| track_ids.contains(&send.target));
        }
        Ok(())
    }

    pub(super) fn configure_mixer_impl(&self, graph: SoundMixerGraph) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        configure_mixer_graph(&mut state, graph)
    }

    pub(super) fn mixer_snapshot_impl(&self) -> Result<SoundMixerSnapshot, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .snapshot())
    }

    pub(super) fn add_or_update_track_impl(
        &self,
        track: SoundTrackDescriptor,
    ) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .add_or_replace_track(track)
    }

    pub(super) fn remove_track_impl(&self, track: SoundTrackId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .remove_track(track)
    }

    pub(super) fn add_or_update_track_send_impl(
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

    pub(super) fn remove_track_send_impl(
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

    pub(super) fn add_or_update_effect_impl(
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

    pub(super) fn remove_effect_impl(
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
