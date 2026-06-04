use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundMixerGraph, SoundMixerGraphManager,
    SoundMixerPresetDescriptor, SoundMixerSnapshot, SoundTrackDescriptor, SoundTrackId,
    SoundTrackSend,
};

use super::super::DefaultSoundManager;

impl SoundMixerGraphManager for DefaultSoundManager {
    fn available_mixer_presets(&self) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError> {
        self.available_mixer_presets_impl()
    }

    fn apply_mixer_preset(&self, locator: &str) -> Result<(), SoundError> {
        self.apply_mixer_preset_impl(locator)
    }

    fn configure_mixer(&self, graph: SoundMixerGraph) -> Result<(), SoundError> {
        self.configure_mixer_impl(graph)
    }

    fn mixer_snapshot(&self) -> Result<SoundMixerSnapshot, SoundError> {
        self.mixer_snapshot_impl()
    }

    fn add_or_update_track(&self, track: SoundTrackDescriptor) -> Result<(), SoundError> {
        self.add_or_update_track_impl(track)
    }

    fn remove_track(&self, track: SoundTrackId) -> Result<(), SoundError> {
        self.remove_track_impl(track)
    }

    fn add_or_update_track_send(
        &self,
        track: SoundTrackId,
        send: SoundTrackSend,
    ) -> Result<(), SoundError> {
        self.add_or_update_track_send_impl(track, send)
    }

    fn remove_track_send(
        &self,
        track: SoundTrackId,
        target: SoundTrackId,
    ) -> Result<(), SoundError> {
        self.remove_track_send_impl(track, target)
    }

    fn add_or_update_effect(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError> {
        self.add_or_update_effect_impl(track, effect)
    }

    fn remove_effect(&self, track: SoundTrackId, effect: SoundEffectId) -> Result<(), SoundError> {
        self.remove_effect_impl(track, effect)
    }
}
