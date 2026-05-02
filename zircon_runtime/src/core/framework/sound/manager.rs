use super::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundBackendStatus, SoundClipId,
    SoundClipInfo, SoundEffectDescriptor, SoundEffectId, SoundError, SoundImpulseResponseId,
    SoundListenerDescriptor, SoundListenerId, SoundMixBlock, SoundMixerGraph, SoundMixerSnapshot,
    SoundParameterId, SoundPlaybackId, SoundPlaybackSettings, SoundSourceDescriptor, SoundSourceId,
    SoundTrackDescriptor, SoundTrackId, SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId,
};

pub trait SoundManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn backend_status(&self) -> SoundBackendStatus;
    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError>;
    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError>;
    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError>;
    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn configure_mixer(&self, graph: SoundMixerGraph) -> Result<(), SoundError>;
    fn mixer_snapshot(&self) -> Result<SoundMixerSnapshot, SoundError>;
    fn add_or_update_track(&self, track: SoundTrackDescriptor) -> Result<(), SoundError>;
    fn remove_track(&self, track: SoundTrackId) -> Result<(), SoundError>;
    fn add_or_update_track_send(
        &self,
        track: SoundTrackId,
        send: SoundTrackSend,
    ) -> Result<(), SoundError>;
    fn remove_track_send(
        &self,
        track: SoundTrackId,
        target: SoundTrackId,
    ) -> Result<(), SoundError>;
    fn add_or_update_effect(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError>;
    fn remove_effect(&self, track: SoundTrackId, effect: SoundEffectId) -> Result<(), SoundError>;
    fn create_source(&self, source: SoundSourceDescriptor) -> Result<SoundSourceId, SoundError>;
    fn update_source(&self, source: SoundSourceDescriptor) -> Result<(), SoundError>;
    fn remove_source(&self, source: SoundSourceId) -> Result<(), SoundError>;
    fn update_listener(&self, listener: SoundListenerDescriptor) -> Result<(), SoundError>;
    fn remove_listener(&self, listener: SoundListenerId) -> Result<(), SoundError>;
    fn update_volume(&self, volume: SoundVolumeDescriptor) -> Result<(), SoundError>;
    fn remove_volume(&self, volume: SoundVolumeId) -> Result<(), SoundError>;
    fn set_parameter(&self, parameter: SoundParameterId, value: f32) -> Result<(), SoundError>;
    fn parameter_value(&self, parameter: &SoundParameterId) -> Result<f32, SoundError>;
    fn bind_automation(&self, binding: SoundAutomationBinding) -> Result<(), SoundError>;
    fn apply_automation_value(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError>;
    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError>;
    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError>;
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError>;
}
