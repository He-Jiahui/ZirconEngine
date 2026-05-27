use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId,
    SoundAutomationCurve, SoundBackendCallbackBlock, SoundBackendCapability, SoundBackendStatus,
    SoundClipId, SoundClipInfo, SoundDynamicEventCatalog, SoundDynamicEventDelivery,
    SoundDynamicEventDescriptor, SoundDynamicEventExecutionReport,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation, SoundEffectDescriptor,
    SoundEffectId, SoundError, SoundExternalSourceBlock, SoundHrtfProfileDescriptor,
    SoundImpulseResponseId, SoundListenerDescriptor, SoundListenerId, SoundMixBlock,
    SoundMixerGraph, SoundMixerPresetDescriptor, SoundMixerSnapshot, SoundOutputDeviceDescriptor,
    SoundOutputDeviceInfo, SoundOutputDeviceStatus, SoundParameterId, SoundPlaybackFinished,
    SoundPlaybackId, SoundPlaybackSettings, SoundPlaybackStatus,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceFinished, SoundSourceId, SoundSourceStatus,
    SoundTimelineSequence, SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
    SoundTrackDescriptor, SoundTrackId, SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId,
};

use super::DefaultSoundManager;
impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {
    fn backend_name(&self) -> String {
        self.backend_name_impl()
    }

    fn backend_status(&self) -> SoundBackendStatus {
        self.backend_status_impl()
    }

    fn configure_output_device(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        self.configure_output_device_impl(descriptor)
    }

    fn start_output_device(&self) -> Result<(), SoundError> {
        self.start_output_device_impl()
    }

    fn stop_output_device(&self) -> Result<(), SoundError> {
        self.stop_output_device_impl()
    }

    fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError> {
        self.output_device_status_impl()
    }

    fn available_output_devices(&self) -> Result<Vec<SoundOutputDeviceInfo>, SoundError> {
        self.available_output_devices_impl()
    }

    fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError> {
        self.render_output_device_block_impl()
    }

    fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError> {
        self.available_output_backends_impl()
    }

    fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError> {
        self.pull_output_backend_callback_impl()
    }

    fn global_volume_gain(&self) -> Result<f32, SoundError> {
        self.global_volume_gain_impl()
    }

    fn set_global_volume_gain(&self, gain: f32) -> Result<(), SoundError> {
        self.set_global_volume_gain_impl(gain)
    }

    fn default_spatial_scale(&self) -> Result<f32, SoundError> {
        self.default_spatial_scale_impl()
    }

    fn set_default_spatial_scale(&self, scale: f32) -> Result<(), SoundError> {
        self.set_default_spatial_scale_impl(scale)
    }

    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        self.load_clip_impl(locator)
    }

    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        self.clip_info_impl(clip)
    }

    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        self.play_clip_impl(clip, settings)
    }

    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.stop_playback_impl(playback)
    }

    fn pause_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.pause_playback_impl(playback)
    }

    fn resume_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.resume_playback_impl(playback)
    }

    fn toggle_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.toggle_playback_impl(playback)
    }

    fn set_playback_gain(&self, playback: SoundPlaybackId, gain: f32) -> Result<(), SoundError> {
        self.set_playback_gain_impl(playback, gain)
    }

    fn set_playback_speed(&self, playback: SoundPlaybackId, speed: f32) -> Result<(), SoundError> {
        self.set_playback_speed_impl(playback, speed)
    }

    fn seek_playback_seconds(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        self.seek_playback_seconds_impl(playback, seconds)
    }

    fn mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.mute_playback_impl(playback)
    }

    fn unmute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.unmute_playback_impl(playback)
    }

    fn toggle_mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.toggle_mute_playback_impl(playback)
    }

    fn playback_empty(&self, playback: SoundPlaybackId) -> Result<bool, SoundError> {
        self.playback_empty_impl(playback)
    }

    fn playback_status(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<SoundPlaybackStatus, SoundError> {
        self.playback_status_impl(playback)
    }

    fn drain_finished_playbacks(&self) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
        self.drain_finished_playbacks_impl()
    }

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

    fn create_source(&self, source: SoundSourceDescriptor) -> Result<SoundSourceId, SoundError> {
        self.create_source_impl(source)
    }

    fn update_source(&self, source: SoundSourceDescriptor) -> Result<(), SoundError> {
        self.update_source_impl(source)
    }

    fn remove_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.remove_source_impl(source)
    }

    fn stop_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.stop_source_impl(source)
    }

    fn pause_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.pause_source_impl(source)
    }

    fn resume_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.resume_source_impl(source)
    }

    fn toggle_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.toggle_source_impl(source)
    }

    fn set_source_gain(&self, source: SoundSourceId, gain: f32) -> Result<(), SoundError> {
        self.set_source_gain_impl(source, gain)
    }

    fn set_source_speed(&self, source: SoundSourceId, speed: f32) -> Result<(), SoundError> {
        self.set_source_speed_impl(source, speed)
    }

    fn seek_source_seconds(&self, source: SoundSourceId, seconds: f32) -> Result<(), SoundError> {
        self.seek_source_seconds_impl(source, seconds)
    }

    fn mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.mute_source_impl(source)
    }

    fn unmute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.unmute_source_impl(source)
    }

    fn toggle_mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.toggle_mute_source_impl(source)
    }

    fn source_empty(&self, source: SoundSourceId) -> Result<bool, SoundError> {
        self.source_empty_impl(source)
    }

    fn source_status(&self, source: SoundSourceId) -> Result<SoundSourceStatus, SoundError> {
        self.source_status_impl(source)
    }

    fn drain_finished_sources(&self) -> Result<Vec<SoundSourceFinished>, SoundError> {
        self.drain_finished_sources_impl()
    }

    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError> {
        self.submit_external_source_block_impl(handle, block)
    }

    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError> {
        self.clear_external_source_impl(handle)
    }

    fn update_listener(&self, listener: SoundListenerDescriptor) -> Result<(), SoundError> {
        self.update_listener_impl(listener)
    }

    fn remove_listener(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        self.remove_listener_impl(listener)
    }

    fn update_volume(&self, volume: SoundVolumeDescriptor) -> Result<(), SoundError> {
        self.update_volume_impl(volume)
    }

    fn remove_volume(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        self.remove_volume_impl(volume)
    }

    fn set_parameter(&self, parameter: SoundParameterId, value: f32) -> Result<(), SoundError> {
        self.set_parameter_impl(parameter, value)
    }

    fn parameter_value(&self, parameter: &SoundParameterId) -> Result<f32, SoundError> {
        self.parameter_value_impl(parameter)
    }

    fn bind_automation(&self, binding: SoundAutomationBinding) -> Result<(), SoundError> {
        self.bind_automation_impl(binding)
    }

    fn apply_automation_value(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        self.apply_automation_value_impl(binding, value)
    }

    fn apply_automation_curve_sample(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        self.apply_automation_curve_sample_impl(binding, curve, time_seconds)
    }

    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError> {
        self.unbind_automation_impl(binding)
    }

    fn schedule_timeline_sequence(
        &self,
        sequence: SoundTimelineSequence,
    ) -> Result<(), SoundError> {
        self.schedule_timeline_sequence_impl(sequence)
    }

    fn remove_timeline_sequence(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError> {
        self.remove_timeline_sequence_impl(sequence)
    }

    fn timeline_sequences(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
        self.timeline_sequences_impl()
    }

    fn advance_timeline_sequences(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        self.advance_timeline_sequences_impl(delta_seconds)
    }

    fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError> {
        self.dynamic_event_catalog_impl()
    }

    fn register_dynamic_event(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError> {
        self.register_dynamic_event_impl(descriptor)
    }

    fn unregister_dynamic_event(&self, event_id: &str) -> Result<(), SoundError> {
        self.unregister_dynamic_event_impl(event_id)
    }

    fn dynamic_event_handlers(
        &self,
    ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
        self.dynamic_event_handlers_impl()
    }

    fn register_dynamic_event_handler(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError> {
        self.register_dynamic_event_handler_impl(handler)
    }

    fn unregister_dynamic_event_handler(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        self.unregister_dynamic_event_handler_impl(plugin_id, handler_id)
    }

    fn submit_dynamic_event(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError> {
        self.submit_dynamic_event_impl(invocation)
    }

    fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
        self.drain_dynamic_events_impl()
    }

    fn dispatch_dynamic_events(&self) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
        self.dispatch_dynamic_events_impl()
    }

    fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError> {
        self.execute_dynamic_events_impl()
    }

    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError> {
        self.set_impulse_response_impl(impulse_response, samples)
    }

    fn remove_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        self.remove_impulse_response_impl(impulse_response)
    }

    fn load_hrtf_profile(&self, profile: SoundHrtfProfileDescriptor) -> Result<(), SoundError> {
        self.load_hrtf_profile_impl(profile)
    }

    fn remove_hrtf_profile(&self, profile_id: &str) -> Result<(), SoundError> {
        self.remove_hrtf_profile_impl(profile_id)
    }

    fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
        self.hrtf_profiles_impl()
    }

    fn set_ray_tracing_convolution_status(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError> {
        self.set_ray_tracing_convolution_status_impl(status)
    }

    fn submit_ray_traced_impulse_response(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError> {
        self.submit_ray_traced_impulse_response_impl(descriptor)
    }

    fn ray_traced_impulse_responses(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError> {
        self.ray_traced_impulse_responses_impl()
    }

    fn clear_ray_traced_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        self.clear_ray_traced_impulse_response_impl(impulse_response)
    }

    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError> {
        self.render_mix_impl(frames)
    }
}
