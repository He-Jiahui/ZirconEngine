use std::sync::{Arc, Mutex};

#[cfg(test)]
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::asset::{AssetUri, ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId,
    SoundAutomationCurve, SoundBackendCallbackBlock, SoundBackendCapability, SoundBackendState,
    SoundBackendStatus, SoundClipId, SoundClipInfo, SoundDynamicEventCatalog,
    SoundDynamicEventDelivery, SoundDynamicEventDescriptor, SoundDynamicEventExecutionReport,
    SoundDynamicEventExecutionStatus, SoundDynamicEventHandlerDescriptor,
    SoundDynamicEventHandlerExecution, SoundDynamicEventInvocation, SoundEffectDescriptor,
    SoundEffectId, SoundError, SoundExternalSourceBlock, SoundHrtfProfileDescriptor,
    SoundImpulseResponseId, SoundListenerDescriptor, SoundListenerId, SoundMixBlock,
    SoundMixerGraph, SoundMixerPresetDescriptor, SoundMixerSnapshot, SoundOutputDeviceDescriptor,
    SoundOutputDeviceStatus, SoundParameterId, SoundPlaybackFinishReason, SoundPlaybackFinished,
    SoundPlaybackId, SoundPlaybackSettings, SoundPlaybackStatus,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished, SoundSourceId,
    SoundSourceInput, SoundSourceStatus, SoundTimelineSequence, SoundTimelineSequenceAdvance,
    SoundTimelineSequenceId, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
    SoundVolumeDescriptor, SoundVolumeId,
};
use zircon_runtime::core::CoreHandle;

use super::automation::{
    apply_automation_target, ensure_finite_value, sample_automation_curve,
    validate_automation_binding,
};
use super::descriptor_validation::{
    validate_external_source_block, validate_external_source_handle,
    validate_hrtf_profile_descriptor, validate_listener_descriptor, validate_source_descriptor,
    validate_volume_descriptor,
};
use super::dynamic_events::{
    dispatch_dynamic_events, register_dynamic_event, register_dynamic_event_handler,
    submit_dynamic_event, unregister_dynamic_event, unregister_dynamic_event_handler,
};
use super::engine::validation::{validate_effect, validate_graph};
use super::engine::{
    ActivePlayback, LoadedClip, SoundDynamicEventExecutor, SoundDynamicEventExecutorKey,
    SoundEngineState, SourceVoice,
};
use super::mixer_configuration::configure_mixer_graph;
use super::output::available_output_backends;
use super::presets::built_in_mixer_presets;
use super::ray_tracing::{
    clear_ray_traced_impulse_response, refresh_ray_tracing_status,
    submit_ray_traced_impulse_response, validate_ray_tracing_status,
};
use super::timeline::{
    advance_timeline_sequences, remove_timeline_sequence, schedule_timeline_sequence,
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

    pub fn register_dynamic_event_executor<F>(
        &self,
        plugin_id: impl Into<String>,
        handler_id: impl Into<String>,
        executor: F,
    ) -> Result<(), SoundError>
    where
        F: Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static,
    {
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if !state.dynamic_event_handlers.iter().any(|handler| {
            handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id
        }) {
            return Err(SoundError::UnknownDynamicEventHandler {
                plugin_id: key.plugin_id,
                handler_id: key.handler_id,
            });
        }
        state
            .dynamic_event_executors
            .insert(key, SoundDynamicEventExecutor::new(executor));
        Ok(())
    }

    pub fn unregister_dynamic_event_executor(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let key = SoundDynamicEventExecutorKey::new(plugin_id, handler_id);
        state
            .dynamic_event_executors
            .remove(&key)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownDynamicEventHandler {
                plugin_id: plugin_id.to_string(),
                handler_id: handler_id.to_string(),
            })
    }
}

fn handler_exists(
    handlers: &[SoundDynamicEventHandlerDescriptor],
    key: &SoundDynamicEventExecutorKey,
) -> bool {
    handlers
        .iter()
        .any(|handler| handler.plugin_id == key.plugin_id && handler.handler_id == key.handler_id)
}

impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {
    fn backend_name(&self) -> String {
        let config = self.config();
        if !config.enabled {
            return "disabled".to_string();
        }
        let unavailable_backend = {
            let state = self.state.lock().expect("sound state mutex poisoned");
            state
                .output_device
                .unavailable_backend_status()
                .map(|(backend, _)| backend.to_string())
        };
        unavailable_backend.unwrap_or(config.backend)
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

        let unavailable_backend = {
            let state = self.state.lock().expect("sound state mutex poisoned");
            state
                .output_device
                .unavailable_backend_status()
                .map(|(backend, detail)| (backend.to_string(), detail.to_string()))
        };
        if let Some((backend, detail)) = unavailable_backend {
            return SoundBackendStatus {
                requested_backend: backend,
                active_backend: None,
                state: SoundBackendState::Unavailable,
                detail: Some(detail),
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
        if let Err(error) = state.output_device.configure(descriptor.clone()) {
            if let SoundError::BackendUnavailable { detail } = &error {
                state
                    .output_device
                    .record_backend_unavailable(descriptor.backend, detail.clone());
            }
            return Err(error);
        }
        config.backend = descriptor.backend.clone();
        config.sample_rate_hz = descriptor.sample_rate_hz;
        config.channel_count = descriptor.channel_count;
        config.block_size_frames = descriptor.block_size_frames;
        state.graph.sample_rate_hz = config.sample_rate_hz;
        state.graph.channel_count = config.channel_count;
        state.effect_states.clear();
        state.track_states.clear();
        state.hrtf_states.clear();
        Ok(())
    }

    fn start_output_device(&self) -> Result<(), SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Some(error) = state.output_device.unavailable_backend_error() {
            return Err(error);
        }
        state.output_device.start();
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

    fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError> {
        Ok(available_output_backends())
    }

    fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError> {
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
                let channel_count = block.channel_count as usize;
                let rendered_frames = if channel_count == 0 {
                    0
                } else {
                    sample_count / channel_count
                };
                let report = state.output_device.record_callback_block(
                    frames,
                    rendered_frames,
                    sample_count,
                );
                Ok(SoundBackendCallbackBlock { report, block })
            }
            Err(error) => {
                state.output_device.record_callback_error(frames, &error);
                Err(error)
            }
        }
    }

    fn global_volume_gain(&self) -> Result<f32, SoundError> {
        Ok(self.config().master_gain)
    }

    fn set_global_volume_gain(&self, gain: f32) -> Result<(), SoundError> {
        ensure_finite_value("global volume gain", gain)?;
        if gain < 0.0 {
            return Err(SoundError::InvalidParameter(
                "global volume gain must be non-negative".to_string(),
            ));
        }
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .master_gain = gain;
        Ok(())
    }

    fn default_spatial_scale(&self) -> Result<f32, SoundError> {
        Ok(self.config().default_spatial_scale)
    }

    fn set_default_spatial_scale(&self, scale: f32) -> Result<(), SoundError> {
        ensure_finite_value("default spatial scale", scale)?;
        if scale < 0.0 {
            return Err(SoundError::InvalidParameter(
                "default spatial scale must be non-negative".to_string(),
            ));
        }
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .default_spatial_scale = scale;
        Ok(())
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
            duration_seconds: clip.asset.duration_seconds(),
        })
    }

    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let loaded_clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        validate_playback_settings(&settings)?;
        let playback_range = playback_range_for_settings(loaded_clip, &settings)?;
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
                cursor_frame: playback_range.start_frame,
                cursor_position: playback_range.start_frame as f64,
                gain: settings.gain,
                speed: validate_playback_speed(settings.speed)?,
                looped: settings.looped,
                completion_action: settings.completion_action,
                paused: settings.paused,
                muted: settings.muted,
                range_start_frame: playback_range.start_frame,
                range_end_frame: playback_range.end_frame,
                output_track: settings.output_track,
                pan: settings.pan,
            },
        );
        Ok(playback_id)
    }

    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .remove(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        state.finished_playbacks.push(SoundPlaybackFinished {
            playback,
            clip: active.clip,
            reason: SoundPlaybackFinishReason::Stopped,
            completion_action: active.completion_action,
            output_track: active.output_track,
        });
        Ok(())
    }

    fn pause_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = true;
        Ok(())
    }

    fn resume_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = false;
        Ok(())
    }

    fn toggle_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.paused = !active.paused;
        Ok(())
    }

    fn set_playback_gain(&self, playback: SoundPlaybackId, gain: f32) -> Result<(), SoundError> {
        ensure_finite_value("playback gain", gain)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.gain = gain;
        Ok(())
    }

    fn set_playback_speed(&self, playback: SoundPlaybackId, speed: f32) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.speed = speed;
        Ok(())
    }

    fn seek_playback_seconds(
        &self,
        playback: SoundPlaybackId,
        seconds: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback seek seconds", seconds)?;
        if seconds < 0.0 {
            return Err(SoundError::InvalidParameter(
                "playback seek seconds must be non-negative".to_string(),
            ));
        }
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let active_clip = active.clip;
        let range_start_frame = active.range_start_frame;
        let range_end_frame = active.range_end_frame;
        let clip = state
            .clips
            .get(&active_clip)
            .ok_or(SoundError::UnknownClip { clip: active_clip })?;
        let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
        let frame_count = clip.asset.frame_count();
        let requested_frame = (seconds * sample_rate).round() as usize;
        let range_end = range_end_frame.unwrap_or(frame_count).min(frame_count);
        let clamped_frame = requested_frame.max(range_start_frame).min(range_end);
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.cursor_frame = clamped_frame;
        active.cursor_position = clamped_frame as f64;
        Ok(())
    }

    fn mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = true;
        Ok(())
    }

    fn unmute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = false;
        Ok(())
    }

    fn toggle_mute_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        active.muted = !active.muted;
        Ok(())
    }

    fn playback_empty(&self, playback: SoundPlaybackId) -> Result<bool, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        if state.playbacks.contains_key(&playback) {
            return Ok(false);
        }
        if state
            .finished_playbacks
            .iter()
            .any(|finished| finished.playback == playback)
        {
            return Ok(true);
        }
        Err(SoundError::UnknownPlayback { playback })
    }

    fn playback_status(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<SoundPlaybackStatus, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let active = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let cursor_seconds = state
            .clips
            .get(&active.clip)
            .map(|clip| {
                if clip.asset.sample_rate_hz == 0 {
                    0.0
                } else {
                    active.cursor_position as f32 / clip.asset.sample_rate_hz as f32
                }
            })
            .unwrap_or_default();
        Ok(SoundPlaybackStatus {
            playback,
            clip: active.clip,
            paused: active.paused,
            muted: active.muted,
            looped: active.looped,
            completion_action: active.completion_action,
            gain: active.gain,
            speed: active.speed,
            range_start_frame: active.range_start_frame,
            range_end_frame: active.range_end_frame,
            cursor_frame: active.cursor_frame,
            cursor_seconds,
            output_track: active.output_track,
        })
    }

    fn drain_finished_playbacks(&self) -> Result<Vec<SoundPlaybackFinished>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        Ok(state.finished_playbacks.drain(..).collect())
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
        state.hrtf_states.clear();
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
                pending_finish: None,
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

    fn stop_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .remove(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        let descriptor = voice.descriptor;
        let input = descriptor.input;
        let clip = match input {
            SoundSourceInput::Clip(clip) => Some(clip),
            SoundSourceInput::External(_)
            | SoundSourceInput::SynthParameter { .. }
            | SoundSourceInput::Silence => None,
        };
        state.finished_sources.push(SoundSourceFinished {
            source,
            input,
            clip,
            reason: SoundSourceFinishReason::Stopped,
            completion_action: descriptor.completion_action,
            output_track: descriptor.output_track,
        });
        Ok(())
    }

    fn pause_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = false;
        Ok(())
    }

    fn resume_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = true;
        Ok(())
    }

    fn toggle_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.playing = !voice.descriptor.playing;
        Ok(())
    }

    fn set_source_gain(&self, source: SoundSourceId, gain: f32) -> Result<(), SoundError> {
        ensure_finite_value("source gain", gain)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.gain = gain;
        Ok(())
    }

    fn set_source_speed(&self, source: SoundSourceId, speed: f32) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.speed = speed;
        Ok(())
    }

    fn seek_source_seconds(&self, source: SoundSourceId, seconds: f32) -> Result<(), SoundError> {
        ensure_finite_value("source seek seconds", seconds)?;
        if seconds < 0.0 {
            return Err(SoundError::InvalidParameter(
                "source seek seconds must be non-negative".to_string(),
            ));
        }
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let clamped_frame = {
            let voice = state
                .sources
                .get(&source)
                .ok_or(SoundError::UnknownSource { source_id: source })?;
            match &voice.descriptor.input {
                SoundSourceInput::Clip(clip_id) => {
                    let clip = state
                        .clips
                        .get(clip_id)
                        .ok_or(SoundError::UnknownClip { clip: *clip_id })?;
                    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
                    let frame_count = clip.asset.frame_count();
                    let start_frame = voice
                        .descriptor
                        .start_seconds
                        .map(|start_seconds| (start_seconds * sample_rate).round() as usize)
                        .unwrap_or_default()
                        .min(frame_count);
                    let range_end = voice
                        .descriptor
                        .duration_seconds
                        .map(|duration_seconds| {
                            let duration_frames =
                                (duration_seconds * sample_rate).round().max(0.0) as usize;
                            start_frame.saturating_add(duration_frames).min(frame_count)
                        })
                        .unwrap_or(frame_count);
                    ((seconds * sample_rate).round() as usize)
                        .max(start_frame)
                        .min(range_end)
                }
                SoundSourceInput::External(handle) => {
                    let block = state.external_sources.get(handle).ok_or_else(|| {
                        SoundError::InvalidParameter(format!(
                            "source seek requires submitted external block for {}",
                            handle.as_str()
                        ))
                    })?;
                    let frame_count = block.samples.len() / block.channel_count.max(1) as usize;
                    ((seconds * block.sample_rate_hz.max(1) as f32).round() as usize)
                        .min(frame_count)
                }
                SoundSourceInput::SynthParameter { .. } | SoundSourceInput::Silence => {
                    if seconds == 0.0 {
                        0
                    } else {
                        return Err(SoundError::InvalidParameter(
                            "source seek requires clip or external input".to_string(),
                        ));
                    }
                }
            }
        };
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.cursor_frame = clamped_frame;
        voice.cursor_position = clamped_frame as f64;
        Ok(())
    }

    fn mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = true;
        Ok(())
    }

    fn unmute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = false;
        Ok(())
    }

    fn toggle_mute_source(&self, source: SoundSourceId) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get_mut(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        voice.descriptor.muted = !voice.descriptor.muted;
        Ok(())
    }

    fn source_empty(&self, source: SoundSourceId) -> Result<bool, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        if state.sources.contains_key(&source) {
            return Ok(false);
        }
        if state
            .finished_sources
            .iter()
            .any(|finished| finished.source == source)
        {
            return Ok(true);
        }
        Err(SoundError::UnknownSource { source_id: source })
    }

    fn source_status(&self, source: SoundSourceId) -> Result<SoundSourceStatus, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let voice = state
            .sources
            .get(&source)
            .ok_or(SoundError::UnknownSource { source_id: source })?;
        let (range_start_frame, range_end_frame, cursor_seconds) =
            source_status_range_and_cursor_seconds(&state, voice);
        Ok(SoundSourceStatus {
            source,
            input: voice.descriptor.input.clone(),
            playing: voice.descriptor.playing,
            muted: voice.descriptor.muted,
            looped: voice.descriptor.looped,
            completion_action: voice.descriptor.completion_action,
            gain: voice.descriptor.gain,
            speed: voice.descriptor.speed,
            range_start_frame,
            range_end_frame,
            cursor_frame: voice.cursor_frame,
            cursor_seconds,
            output_track: voice.descriptor.output_track,
        })
    }

    fn drain_finished_sources(&self) -> Result<Vec<SoundSourceFinished>, SoundError> {
        Ok(std::mem::take(
            &mut self
                .state
                .lock()
                .expect("sound state mutex poisoned")
                .finished_sources,
        ))
    }

    fn submit_external_source_block(
        &self,
        handle: ExternalAudioSourceHandle,
        block: SoundExternalSourceBlock,
    ) -> Result<(), SoundError> {
        validate_external_source_handle(&handle)?;
        validate_external_source_block(&block)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .external_sources
            .insert(handle, block);
        Ok(())
    }

    fn clear_external_source(&self, handle: &ExternalAudioSourceHandle) -> Result<(), SoundError> {
        validate_external_source_handle(handle)?;
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

    fn schedule_timeline_sequence(
        &self,
        sequence: SoundTimelineSequence,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        schedule_timeline_sequence(&mut state, sequence)
    }

    fn remove_timeline_sequence(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        remove_timeline_sequence(&mut state, sequence)
    }

    fn timeline_sequences(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .timeline_sequences
            .iter()
            .map(|playback| playback.sequence.clone())
            .collect())
    }

    fn advance_timeline_sequences(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        advance_timeline_sequences(&mut state, delta_seconds)
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
            .dynamic_event_handlers
            .retain(|handler| handler.event_id != event_id);
        let handlers = state.dynamic_event_handlers.clone();
        state
            .dynamic_event_executors
            .retain(|key, _| handler_exists(&handlers, key));
        state
            .pending_dynamic_events
            .retain(|event| event.event_id != event_id);
        Ok(())
    }

    fn dynamic_event_handlers(
        &self,
    ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .dynamic_event_handlers
            .clone())
    }

    fn register_dynamic_event_handler(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let catalog = state.dynamic_events.clone();
        register_dynamic_event_handler(&catalog, &mut state.dynamic_event_handlers, handler)
    }

    fn unregister_dynamic_event_handler(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        unregister_dynamic_event_handler(&mut state.dynamic_event_handlers, plugin_id, handler_id)?;
        state
            .dynamic_event_executors
            .remove(&SoundDynamicEventExecutorKey::new(plugin_id, handler_id));
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

    fn dispatch_dynamic_events(&self) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let handlers = state.dynamic_event_handlers.clone();
        Ok(dispatch_dynamic_events(
            &handlers,
            &mut state.pending_dynamic_events,
        ))
    }

    fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError> {
        let (deliveries, executors) = {
            let mut state = self.state.lock().expect("sound state mutex poisoned");
            let handlers = state.dynamic_event_handlers.clone();
            let deliveries = dispatch_dynamic_events(&handlers, &mut state.pending_dynamic_events);
            (deliveries, state.dynamic_event_executors.clone())
        };
        let executions = deliveries
            .into_iter()
            .map(|delivery| {
                let key = SoundDynamicEventExecutorKey::from_handler(&delivery.handler);
                match executors.get(&key) {
                    Some(executor) => match executor.execute(&delivery) {
                        Ok(()) => SoundDynamicEventHandlerExecution {
                            delivery,
                            status: SoundDynamicEventExecutionStatus::Succeeded,
                            detail: None,
                        },
                        Err(detail) => SoundDynamicEventHandlerExecution {
                            delivery,
                            status: SoundDynamicEventExecutionStatus::Failed,
                            detail: Some(detail),
                        },
                    },
                    None => SoundDynamicEventHandlerExecution {
                        delivery,
                        status: SoundDynamicEventExecutionStatus::SkippedMissingExecutor,
                        detail: None,
                    },
                }
            })
            .collect();
        Ok(SoundDynamicEventExecutionReport { executions })
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

    fn load_hrtf_profile(&self, profile: SoundHrtfProfileDescriptor) -> Result<(), SoundError> {
        validate_hrtf_profile_descriptor(&profile)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .insert(profile.profile_id.clone(), profile);
        state.hrtf_states.clear();
        Ok(())
    }

    fn remove_hrtf_profile(&self, profile_id: &str) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .remove(profile_id)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownHrtfProfile {
                profile_id: profile_id.to_string(),
            })?;
        state.hrtf_states.clear();
        Ok(())
    }

    fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
        let mut profiles = self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .hrtf_profiles
            .values()
            .cloned()
            .collect::<Vec<_>>();
        profiles.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(profiles)
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

fn validate_playback_speed(speed: f32) -> Result<f32, SoundError> {
    ensure_finite_value("playback speed", speed)?;
    if speed <= 0.0 {
        return Err(SoundError::InvalidParameter(
            "playback speed must be greater than zero".to_string(),
        ));
    }
    Ok(speed)
}

fn validate_playback_settings(settings: &SoundPlaybackSettings) -> Result<(), SoundError> {
    ensure_finite_value("playback gain", settings.gain)?;
    ensure_finite_value("playback pan", settings.pan)?;
    validate_playback_speed(settings.speed)?;
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct PlaybackRange {
    start_frame: usize,
    end_frame: Option<usize>,
}

fn playback_range_for_settings(
    clip: &LoadedClip,
    settings: &SoundPlaybackSettings,
) -> Result<PlaybackRange, SoundError> {
    let frame_count = clip.asset.frame_count();
    let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
    let start_frame = seconds_to_frame(
        "playback start seconds",
        settings.start_seconds,
        sample_rate,
    )?
    .unwrap_or_default()
    .min(frame_count);
    let end_frame = seconds_to_frame(
        "playback duration seconds",
        settings.duration_seconds,
        sample_rate,
    )?
    .map(|duration_frames| start_frame.saturating_add(duration_frames).min(frame_count));
    if matches!(end_frame, Some(end) if end <= start_frame) {
        return Err(SoundError::InvalidParameter(
            "playback duration must cover at least one frame".to_string(),
        ));
    }
    Ok(PlaybackRange {
        start_frame,
        end_frame,
    })
}

fn seconds_to_frame(
    label: &str,
    seconds: Option<f32>,
    sample_rate: f32,
) -> Result<Option<usize>, SoundError> {
    let Some(seconds) = seconds else {
        return Ok(None);
    };
    ensure_finite_value(label, seconds)?;
    if seconds < 0.0 {
        return Err(SoundError::InvalidParameter(format!(
            "{label} must be non-negative"
        )));
    }
    Ok(Some((seconds * sample_rate).round() as usize))
}

fn source_status_range_and_cursor_seconds(
    state: &SoundEngineState,
    voice: &SourceVoice,
) -> (usize, Option<usize>, f32) {
    match &voice.descriptor.input {
        SoundSourceInput::Clip(clip_id) => {
            let Some(clip) = state.clips.get(clip_id) else {
                return (0, None, 0.0);
            };
            let sample_rate = clip.asset.sample_rate_hz.max(1) as f32;
            let frame_count = clip.asset.frame_count();
            let start_frame = voice
                .descriptor
                .start_seconds
                .map(|seconds| (seconds * sample_rate).round().max(0.0) as usize)
                .unwrap_or_default()
                .min(frame_count);
            let end_frame = voice.descriptor.duration_seconds.map(|seconds| {
                let duration_frames = (seconds * sample_rate).round().max(0.0) as usize;
                start_frame.saturating_add(duration_frames).min(frame_count)
            });
            (
                start_frame,
                end_frame,
                voice.cursor_position as f32 / sample_rate,
            )
        }
        SoundSourceInput::External(handle) => {
            let sample_rate = state
                .external_sources
                .get(handle)
                .map(|block| block.sample_rate_hz.max(1) as f32)
                .unwrap_or(1.0);
            (0, None, voice.cursor_position as f32 / sample_rate)
        }
        SoundSourceInput::SynthParameter { .. } | SoundSourceInput::Silence => (0, None, 0.0),
    }
}
