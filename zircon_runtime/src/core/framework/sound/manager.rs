use super::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId,
    SoundAutomationCurve, SoundBackendStatus, SoundClipId, SoundClipInfo, SoundEffectDescriptor,
    SoundEffectId, SoundError, SoundExternalSourceBlock, SoundImpulseResponseId,
    SoundListenerDescriptor, SoundListenerId, SoundMixBlock, SoundMixerGraph,
    SoundMixerPresetDescriptor, SoundMixerSnapshot, SoundOutputDeviceDescriptor,
    SoundOutputDeviceStatus, SoundParameterId, SoundPlaybackId, SoundPlaybackSettings,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
    SoundVolumeDescriptor, SoundVolumeId,
};
use super::{SoundDynamicEventCatalog, SoundDynamicEventDescriptor, SoundDynamicEventInvocation};

pub trait SoundManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn backend_status(&self) -> SoundBackendStatus;
    fn configure_output_device(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError>;
    fn start_output_device(&self) -> Result<(), SoundError>;
    fn stop_output_device(&self) -> Result<(), SoundError>;
    fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError>;
    fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError>;
    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError>;
    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError>;
    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError>;
    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError>;
    fn available_mixer_presets(&self) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError>;
    fn apply_mixer_preset(&self, locator: &str) -> Result<(), SoundError>;
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
    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError>;
    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError>;
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
    fn apply_automation_curve_sample(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError>;
    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError>;
    fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError>;
    fn register_dynamic_event(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError>;
    fn unregister_dynamic_event(&self, event_id: &str) -> Result<(), SoundError>;
    fn submit_dynamic_event(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError>;
    fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError>;
    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError>;
    fn remove_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError>;
    fn set_ray_tracing_convolution_status(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError>;
    fn submit_ray_traced_impulse_response(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError>;
    fn ray_traced_impulse_responses(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError>;
    fn clear_ray_traced_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError>;
    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError>;
}
