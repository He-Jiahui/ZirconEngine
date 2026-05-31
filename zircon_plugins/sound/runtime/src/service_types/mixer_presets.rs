use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundMixerPresetDescriptor, SoundTrackId, SoundTrackMeter,
};

use crate::engine::validation::validate_graph;
use crate::presets::catalog::built_in_mixer_presets;

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
}
