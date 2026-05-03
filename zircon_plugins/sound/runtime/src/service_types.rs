use std::sync::{Arc, Mutex};

#[cfg(test)]
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::asset::{AssetUri, ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId,
    SoundAutomationCurve, SoundBackendState, SoundBackendStatus, SoundClipId, SoundClipInfo,
    SoundDynamicEventCatalog, SoundDynamicEventDescriptor, SoundDynamicEventInvocation,
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundExternalSourceBlock,
    SoundImpulseResponseId, SoundListenerDescriptor, SoundListenerId, SoundMixBlock,
    SoundMixerGraph, SoundMixerPresetDescriptor, SoundMixerSnapshot, SoundOutputDeviceDescriptor,
    SoundOutputDeviceStatus, SoundParameterId, SoundPlaybackId, SoundPlaybackSettings,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
    SoundVolumeDescriptor, SoundVolumeId,
};
use zircon_runtime::core::CoreHandle;

use super::automation::{
    apply_automation_target, ensure_finite_value, sample_automation_curve,
    validate_automation_binding,
};
use super::descriptor_validation::{
    validate_external_source_block, validate_listener_descriptor, validate_source_descriptor,
    validate_volume_descriptor,
};
use super::dynamic_events::{
    register_dynamic_event, submit_dynamic_event, unregister_dynamic_event,
};
use super::engine::validation::{validate_effect, validate_graph};
use super::engine::{ActivePlayback, LoadedClip, SoundEngineState, SourceVoice};
use super::mixer_configuration::configure_mixer_graph;
use super::presets::built_in_mixer_presets;
use super::ray_tracing::{
    clear_ray_traced_impulse_response, refresh_ray_tracing_status,
    submit_ray_traced_impulse_response, validate_ray_tracing_status,
};
use super::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

#[derive(Clone, Debug)]
pub struct DefaultSoundManager {
    core: Option<CoreHandle>,
    config: Arc<Mutex<SoundConfig>>,
    state: Arc<Mutex<SoundEngineState>>,
}

impl Default for DefaultSoundManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSoundManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let config = SoundConfig::default();
        Self {
            core,
            config: Arc::new(Mutex::new(config.clone())),
            state: Arc::new(Mutex::new(SoundEngineState::new(&config))),
        }
    }

    fn config(&self) -> SoundConfig {
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .clone()
    }

    fn project_asset_manager(&self) -> Result<Arc<ProjectAssetManager>, SoundError> {
        let core = self
            .core
            .as_ref()
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: "sound manager is not attached to a CoreRuntime".to_string(),
            })?;
        core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
            .map_err(|error| SoundError::BackendUnavailable {
                detail: error.to_string(),
            })
    }

    #[cfg(test)]
    pub(crate) fn insert_clip_for_test(&self, asset: SoundAsset) -> SoundClipId {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state.clips.insert(clip_id, LoadedClip { asset });
        clip_id
    }
}

impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {
    fn backend_name(&self) -> String {
        let config = self.config();
        if config.enabled {
            config.backend
        } else {
            "disabled".to_string()
        }
    }

    fn backend_status(&self) -> SoundBackendStatus {
        let config = self.config();
        if !config.enabled {
            return SoundBackendStatus {
                requested_backend: config.backend,
                active_backend: None,
                state: SoundBackendState::Disabled,
                detail: Some("sound playback is disabled".to_string()),
                sample_rate_hz: config.sample_rate_hz,
                channel_count: config.channel_count,
            };
        }

        SoundBackendStatus {
            requested_backend: config.backend.clone(),
            active_backend: Some(config.backend),
            state: SoundBackendState::Ready,
            detail: None,
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
        }
    }

    fn configure_output_device(
        &self,
        descriptor: SoundOutputDeviceDescriptor,
    ) -> Result<(), SoundError> {
        let mut config = self.config.lock().expect("sound config mutex poisoned");
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state.output_device.configure(descriptor.clone())?;
        config.backend = descriptor.backend.clone();
        config.sample_rate_hz = descriptor.sample_rate_hz;
        config.channel_count = descriptor.channel_count;
        config.block_size_frames = descriptor.block_size_frames;
        state.graph.sample_rate_hz = config.sample_rate_hz;
        state.graph.channel_count = config.channel_count;
        state.effect_states.clear();
        state.track_states.clear();
        Ok(())
    }

    fn start_output_device(&self) -> Result<(), SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .start();
        Ok(())
    }

    fn stop_output_device(&self) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .stop();
        Ok(())
    }

    fn output_device_status(&self) -> Result<SoundOutputDeviceStatus, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .status())
    }

    fn render_output_device_block(&self) -> Result<SoundMixBlock, SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let frames = state.output_device.block_size_frames()?;
        match state.render_mix(&config, frames) {
            Ok(block) => {
                let sample_count = block.samples.len();
                state
                    .output_device
                    .record_rendered_block(frames, sample_count);
                Ok(block)
            }
            Err(error) => {
                state.output_device.record_error(&error);
                Err(error)
            }
        }
    }

    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        let uri = AssetUri::parse(locator).map_err(|_| SoundError::InvalidLocator {
            locator: locator.to_string(),
        })?;
        let asset_manager = self.project_asset_manager()?;
        let asset_id =
            asset_manager
                .resolve_asset_id(&uri)
                .ok_or_else(|| SoundError::InvalidLocator {
                    locator: locator.to_string(),
                })?;
        let asset = asset_manager
            .load_sound_asset(asset_id)
            .map_err(|error| SoundError::Decode(error.to_string()))?;

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Some(existing) = state.clip_ids_by_locator.get(locator).copied() {
            return Ok(existing);
        }

        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state
            .clip_ids_by_locator
            .insert(locator.to_string(), clip_id);
        state.clips.insert(clip_id, LoadedClip { asset });
        Ok(clip_id)
    }

    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        Ok(SoundClipInfo {
            locator: clip.asset.uri.to_string(),
            sample_rate_hz: clip.asset.sample_rate_hz,
            channel_count: clip.asset.channel_count,
            frame_count: clip.asset.frame_count(),
            duration_seconds: clip.asset.frame_count() as f32 / clip.asset.sample_rate_hz as f32,
        })
    }

    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if !state.clips.contains_key(&clip) {
            return Err(SoundError::UnknownClip { clip });
        }
        if !state
            .graph
            .tracks
            .iter()
            .any(|track| track.id == settings.output_track)
        {
            return Err(SoundError::UnknownTrack {
                track: settings.output_track,
            });
        }

        state.next_playback_id += 1;
        let playback_id = SoundPlaybackId::new(state.next_playback_id);
        state.playbacks.insert(
            playback_id,
            ActivePlayback {
                clip,
                cursor_frame: 0,
                cursor_position: 0.0,
                gain: settings.gain,
                looped: settings.looped,
                output_track: settings.output_track,
                pan: settings.pan,
            },
        );
        Ok(playback_id)
    }

    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .playbacks
            .remove(&playback)
            .map(|_| ())
            .ok_or(SoundError::UnknownPlayback { playback })
    }

    fn available_mixer_presets(&self) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError> {
        Ok(built_in_mixer_presets(&self.config()))
    }

    fn apply_mixer_preset(&self, locator: &str) -> Result<(), SoundError> {
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
        state.meters = vec![
            zircon_runtime::core::framework::sound::SoundTrackMeter::silent(SoundTrackId::master()),
        ];
        let track_ids = state
            .graph
            .tracks
            .iter()
            .map(|track| track.id)
            .collect::<std::collections::HashSet<_>>();
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

    fn configure_mixer(&self, graph: SoundMixerGraph) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        configure_mixer_graph(&mut state, graph)
    }

    fn mixer_snapshot(&self) -> Result<SoundMixerSnapshot, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .snapshot())
    }

    fn add_or_update_track(&self, track: SoundTrackDescriptor) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .add_or_replace_track(track)
    }

    fn remove_track(&self, track: SoundTrackId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .remove_track(track)
    }

    fn add_or_update_track_send(
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

    fn remove_track_send(
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

    fn add_or_update_effect(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError> {
        validate_effect(&effect)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let track = state
            .graph
            .tracks
            .iter_mut()
            .find(|candidate| candidate.id == track)
            .ok_or(SoundError::UnknownTrack { track })?;
        if let Some(existing) = track
            .effects
            .iter_mut()
            .find(|candidate| candidate.id == effect.id)
        {
            *existing = effect;
        } else {
            track.effects.push(effect);
        }
        Ok(())
    }

    fn remove_effect(&self, track: SoundTrackId, effect: SoundEffectId) -> Result<(), SoundError> {
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

    fn create_source(
        &self,
        mut source: SoundSourceDescriptor,
    ) -> Result<SoundSourceId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_source_descriptor(&state, &source)?;
        if !state
            .graph
            .tracks
            .iter()
            .any(|track| track.id == source.output_track)
        {
            return Err(SoundError::UnknownTrack {
                track: source.output_track,
            });
        }
        let source_id = source.id.unwrap_or_else(|| state.next_source_id());
        source.id = Some(source_id);
        state.sources.insert(
            source_id,
            SourceVoice {
                descriptor: source,
                cursor_frame: 0,
                cursor_position: 0.0,
            },
        );
        Ok(source_id)
    }

    fn update_source(&self, source: SoundSourceDescriptor) -> Result<(), SoundError> {
        let source_id = source.id.ok_or_else(|| {
            SoundError::InvalidParameter("source update requires a source id".to_string())
        })?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_source_descriptor(&state, &source)?;
        let voice = state
            .sources
            .get_mut(&source_id)
            .ok_or(SoundError::UnknownSource { source_id })?;
        voice.descriptor = source;
        Ok(())
    }

    fn remove_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .sources
            .remove(&source)
            .map(|_| ())
            .ok_or(SoundError::UnknownSource { source_id: source })
    }

    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError> {
        validate_external_source_block(&block)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .external_sources
            .insert(handle, block);
        Ok(())
    }

    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .external_sources
            .remove(handle)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownExternalSource {
                handle: handle.clone(),
            })
    }

    fn update_listener(&self, listener: SoundListenerDescriptor) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_listener_descriptor(&state, &listener)?;
        state.listeners.insert(listener.id, listener);
        Ok(())
    }

    fn remove_listener(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .listeners
            .remove(&listener)
            .map(|_| ())
            .ok_or(SoundError::UnknownListener { listener })
    }

    fn update_volume(&self, volume: SoundVolumeDescriptor) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_volume_descriptor(&volume)?;
        state.volumes.insert(volume.id, volume);
        Ok(())
    }

    fn remove_volume(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .volumes
            .remove(&volume)
            .map(|_| ())
            .ok_or(SoundError::UnknownVolume { volume })
    }

    fn set_parameter(&self, parameter: SoundParameterId, value: f32) -> Result<(), SoundError> {
        if !value.is_finite() {
            return Err(SoundError::InvalidParameter(format!(
                "parameter {} must be finite",
                parameter.as_str()
            )));
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .insert(parameter, value);
        Ok(())
    }

    fn parameter_value(&self, parameter: &SoundParameterId) -> Result<f32, SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .get(parameter)
            .copied()
            .ok_or_else(|| SoundError::UnknownParameter {
                parameter: parameter.clone(),
            })
    }

    fn bind_automation(&self, binding: SoundAutomationBinding) -> Result<(), SoundError> {
        validate_automation_binding(&binding)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .automation_bindings
            .insert(binding.id, binding);
        Ok(())
    }

    fn apply_automation_value(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("automation value", value)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let binding_descriptor = state
            .automation_bindings
            .get(&binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding { binding })?;
        apply_automation_target(
            &mut state,
            binding_descriptor.target,
            &binding_descriptor.parameter,
            value,
        )
    }

    fn apply_automation_curve_sample(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        let value = sample_automation_curve(curve, time_seconds)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let binding_descriptor = state
            .automation_bindings
            .get(&binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding { binding })?;
        apply_automation_target(
            &mut state,
            binding_descriptor.target,
            &binding_descriptor.parameter,
            value,
        )?;
        Ok(value)
    }

    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .automation_bindings
            .remove(&binding)
            .map(|_| ())
            .ok_or(SoundError::UnknownAutomationBinding { binding })
    }

    fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .dynamic_events
            .clone())
    }

    fn register_dynamic_event(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        register_dynamic_event(&mut state.dynamic_events, descriptor)
    }

    fn unregister_dynamic_event(&self, event_id: &str) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        unregister_dynamic_event(&mut state.dynamic_events, event_id)?;
        state
            .pending_dynamic_events
            .retain(|event| event.event_id != event_id);
        Ok(())
    }

    fn submit_dynamic_event(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let catalog = state.dynamic_events.clone();
        submit_dynamic_event(&catalog, &mut state.pending_dynamic_events, invocation)
    }

    fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        Ok(state.pending_dynamic_events.drain(..).collect())
    }

    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError> {
        if samples.is_empty() || samples.iter().any(|sample| !sample.is_finite()) {
            return Err(SoundError::InvalidParameter(
                "impulse response samples must be non-empty and finite".to_string(),
            ));
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .impulse_responses
            .insert(impulse_response, samples);
        Ok(())
    }

    fn remove_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .impulse_responses
            .remove(&impulse_response)
            .map(|_| ())
            .ok_or(SoundError::UnknownImpulseResponse { impulse_response })?;
        state.ray_traced_impulse_responses.remove(&impulse_response);
        let descriptors = state.ray_traced_impulse_responses.clone();
        refresh_ray_tracing_status(&mut state.ray_tracing, &descriptors);
        Ok(())
    }

    fn set_ray_tracing_convolution_status(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError> {
        validate_ray_tracing_status(&status)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .ray_tracing = status;
        Ok(())
    }

    fn submit_ray_traced_impulse_response(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        submit_ray_traced_impulse_response(&mut state, descriptor)
    }

    fn ray_traced_impulse_responses(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .ray_traced_impulse_responses
            .values()
            .cloned()
            .collect())
    }

    fn clear_ray_traced_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        clear_ray_traced_impulse_response(&mut state, impulse_response)
    }

    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .render_mix(&config, frames)
    }
}
