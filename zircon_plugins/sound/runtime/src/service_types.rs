use std::sync::{Arc, Mutex};

#[cfg(test)]
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::asset::{AssetUri, ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationTarget, SoundBackendState,
    SoundBackendStatus, SoundClipId, SoundClipInfo, SoundEffectDescriptor, SoundEffectId,
    SoundEffectKind, SoundError, SoundImpulseResponseId, SoundListenerDescriptor, SoundListenerId,
    SoundMixBlock, SoundMixerGraph, SoundMixerSnapshot, SoundParameterId, SoundPlaybackId,
    SoundPlaybackSettings, SoundSourceDescriptor, SoundSourceId, SoundTrackDescriptor,
    SoundTrackId, SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId,
};
use zircon_runtime::core::CoreHandle;

use super::engine::validation::{validate_effect, validate_graph};
use super::engine::{ActivePlayback, LoadedClip, SoundEngineState, SourceVoice};
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

    fn configure_mixer(&self, graph: SoundMixerGraph) -> Result<(), SoundError> {
        validate_graph(&graph)?;
        self.state.lock().expect("sound state mutex poisoned").graph = graph;
        Ok(())
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
            .ok_or_else(|| {
                SoundError::InvalidParameter(format!("unknown listener {}", listener.raw()))
            })
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
            .ok_or_else(|| SoundError::InvalidParameter(format!("unknown volume {}", volume.raw())))
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
        if binding.timeline_track_path.trim().is_empty() {
            return Err(SoundError::InvalidParameter(
                "automation binding requires a timeline track path".to_string(),
            ));
        }
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

    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .automation_bindings
            .remove(&binding)
            .map(|_| ())
            .ok_or(SoundError::UnknownAutomationBinding { binding })
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

fn apply_automation_target(
    state: &mut SoundEngineState,
    target: SoundAutomationTarget,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match target {
        SoundAutomationTarget::Track(track) => {
            let mut graph = state.graph.clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            apply_track_parameter(track_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.graph = graph;
            Ok(())
        }
        SoundAutomationTarget::Effect { track, effect } => {
            let mut graph = state.graph.clone();
            let track_descriptor = graph
                .tracks
                .iter_mut()
                .find(|candidate| candidate.id == track)
                .ok_or(SoundError::UnknownTrack { track })?;
            let effect_descriptor = track_descriptor
                .effects
                .iter_mut()
                .find(|candidate| candidate.id == effect)
                .ok_or(SoundError::UnknownEffect { effect })?;
            apply_effect_parameter(effect_descriptor, parameter, value)?;
            validate_graph(&graph)?;
            state.graph = graph;
            Ok(())
        }
        SoundAutomationTarget::Source(source_id) => {
            let mut descriptor = state
                .sources
                .get(&source_id)
                .ok_or(SoundError::UnknownSource { source_id })?
                .descriptor
                .clone();
            apply_source_parameter(&mut descriptor, parameter, value)?;
            validate_source_descriptor(state, &descriptor)?;
            state
                .sources
                .get_mut(&source_id)
                .expect("validated source disappeared")
                .descriptor = descriptor;
            Ok(())
        }
        SoundAutomationTarget::Listener(listener) => {
            let mut descriptor = state
                .listeners
                .get(&listener)
                .ok_or(SoundError::UnknownListener { listener })?
                .clone();
            apply_listener_parameter(&mut descriptor, parameter, value)?;
            validate_listener_descriptor(state, &descriptor)?;
            state.listeners.insert(listener, descriptor);
            Ok(())
        }
        SoundAutomationTarget::Volume(volume) => {
            let mut descriptor = state
                .volumes
                .get(&volume)
                .ok_or(SoundError::UnknownVolume { volume })?
                .clone();
            apply_volume_parameter(&mut descriptor, parameter, value)?;
            validate_volume_descriptor(&descriptor)?;
            state.volumes.insert(volume, descriptor);
            Ok(())
        }
        SoundAutomationTarget::SynthParameter(target_parameter) => {
            if parameter.as_str() != "value" && parameter.as_str() != target_parameter.as_str() {
                return Err(unsupported_automation_parameter(
                    "synth parameter",
                    parameter,
                ));
            }
            state.parameters.insert(target_parameter, value);
            Ok(())
        }
    }
}

fn apply_track_parameter(
    track: &mut SoundTrackDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => track.controls.gain = value,
        "pan" => track.controls.pan = value,
        "left_gain" => track.controls.left_gain = value,
        "right_gain" => track.controls.right_gain = value,
        "delay_frames" => track.controls.delay_frames = non_negative_usize(parameter, value)?,
        "invert_left_phase" => track.controls.invert_left_phase = bool_from_value(value),
        "invert_right_phase" => track.controls.invert_right_phase = bool_from_value(value),
        "mute" => track.controls.mute = bool_from_value(value),
        "solo" => track.controls.solo = bool_from_value(value),
        "bypass_effects" => track.controls.bypass_effects = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("track", parameter)),
    }
    Ok(())
}

fn apply_effect_parameter(
    effect: &mut SoundEffectDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "enabled" => {
            effect.enabled = bool_from_value(value);
            return Ok(());
        }
        "bypass" => {
            effect.bypass = bool_from_value(value);
            return Ok(());
        }
        "wet" => {
            effect.wet = value;
            validate_effect(effect)?;
            return Ok(());
        }
        _ => {}
    }

    match &mut effect.kind {
        SoundEffectKind::Gain(gain) => match parameter.as_str() {
            "gain" => gain.gain = value,
            _ => return Err(unsupported_automation_parameter("gain effect", parameter)),
        },
        SoundEffectKind::Filter(filter) => match parameter.as_str() {
            "cutoff_hz" => filter.cutoff_hz = value,
            "resonance" => filter.resonance = value,
            "gain_db" => filter.gain_db = value,
            _ => return Err(unsupported_automation_parameter("filter effect", parameter)),
        },
        SoundEffectKind::Reverb(reverb) => match parameter.as_str() {
            "room_size" => reverb.room_size = value,
            "damping" => reverb.damping = value,
            "pre_delay_frames" => {
                reverb.pre_delay_frames = non_negative_usize(parameter, value)?;
            }
            "tail_frames" => reverb.tail_frames = non_negative_usize(parameter, value)?,
            _ => return Err(unsupported_automation_parameter("reverb effect", parameter)),
        },
        SoundEffectKind::ConvolutionReverb(convolution) => match parameter.as_str() {
            "latency_frames" => {
                convolution.latency_frames = non_negative_usize(parameter, value)?;
            }
            "fallback_to_algorithmic" => convolution.fallback_to_algorithmic = bool_from_value(value),
            _ => {
                return Err(unsupported_automation_parameter(
                    "convolution reverb effect",
                    parameter,
                ));
            }
        },
        SoundEffectKind::Compressor(compressor) => match parameter.as_str() {
            "threshold_db" => compressor.threshold_db = value,
            "ratio" => compressor.ratio = value,
            "attack_ms" => compressor.attack_ms = value,
            "release_ms" => compressor.release_ms = value,
            "makeup_gain_db" => compressor.makeup_gain_db = value,
            _ => return Err(unsupported_automation_parameter("compressor effect", parameter)),
        },
        SoundEffectKind::WaveShaper(shaper) => match parameter.as_str() {
            "drive" => shaper.drive = value,
            _ => return Err(unsupported_automation_parameter("wave shaper effect", parameter)),
        },
        SoundEffectKind::Flanger(flanger) => match parameter.as_str() {
            "delay_frames" => flanger.delay_frames = non_negative_usize(parameter, value)?,
            "depth_frames" => flanger.depth_frames = non_negative_usize(parameter, value)?,
            "rate_hz" => flanger.rate_hz = value,
            "feedback" => flanger.feedback = value,
            _ => return Err(unsupported_automation_parameter("flanger effect", parameter)),
        },
        SoundEffectKind::Phaser(phaser) => match parameter.as_str() {
            "rate_hz" => phaser.rate_hz = value,
            "depth" => phaser.depth = value,
            "feedback" => phaser.feedback = value,
            "phase_offset" => phaser.phase_offset = value,
            _ => return Err(unsupported_automation_parameter("phaser effect", parameter)),
        },
        SoundEffectKind::Chorus(chorus) => match parameter.as_str() {
            "voices" => chorus.voices = u8_from_value(parameter, value)?,
            "delay_frames" => chorus.delay_frames = non_negative_usize(parameter, value)?,
            "depth_frames" => chorus.depth_frames = non_negative_usize(parameter, value)?,
            "rate_hz" => chorus.rate_hz = value,
            _ => return Err(unsupported_automation_parameter("chorus effect", parameter)),
        },
        SoundEffectKind::Delay(delay) => match parameter.as_str() {
            "delay_frames" => delay.delay_frames = non_negative_usize(parameter, value)?,
            "feedback" => delay.feedback = value,
            _ => return Err(unsupported_automation_parameter("delay effect", parameter)),
        },
        SoundEffectKind::PanStereo(pan) => match parameter.as_str() {
            "pan" => pan.pan = value,
            "width" => pan.width = value,
            "left_gain" => pan.left_gain = value,
            "right_gain" => pan.right_gain = value,
            "invert_left_phase" => pan.invert_left_phase = bool_from_value(value),
            "invert_right_phase" => pan.invert_right_phase = bool_from_value(value),
            _ => return Err(unsupported_automation_parameter("pan stereo effect", parameter)),
        },
        SoundEffectKind::Limiter(limiter) => match parameter.as_str() {
            "ceiling" => limiter.ceiling = value,
            _ => return Err(unsupported_automation_parameter("limiter effect", parameter)),
        },
    }
    validate_effect(effect)
}

fn apply_source_parameter(
    source: &mut SoundSourceDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => source.gain = value,
        "playing" => source.playing = bool_from_value(value),
        "looped" => source.looped = bool_from_value(value),
        "position_x" => source.position[0] = value,
        "position_y" => source.position[1] = value,
        "position_z" => source.position[2] = value,
        "forward_x" => source.forward[0] = value,
        "forward_y" => source.forward[1] = value,
        "forward_z" => source.forward[2] = value,
        "velocity_x" => source.velocity[0] = value,
        "velocity_y" => source.velocity[1] = value,
        "velocity_z" => source.velocity[2] = value,
        "spatial_blend" => source.spatial.spatial_blend = value,
        "min_distance" => source.spatial.min_distance = value,
        "max_distance" => source.spatial.max_distance = value,
        "cone_inner_degrees" => source.spatial.cone_inner_degrees = value,
        "cone_outer_degrees" => source.spatial.cone_outer_degrees = value,
        "doppler_factor" => source.spatial.doppler_factor = value,
        "occlusion_enabled" => source.spatial.occlusion_enabled = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("source", parameter)),
    }
    Ok(())
}

fn apply_listener_parameter(
    listener: &mut SoundListenerDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "active" => listener.active = bool_from_value(value),
        "doppler_tracking" => listener.doppler_tracking = bool_from_value(value),
        "position_x" => listener.position[0] = value,
        "position_y" => listener.position[1] = value,
        "position_z" => listener.position[2] = value,
        "forward_x" => listener.forward[0] = value,
        "forward_y" => listener.forward[1] = value,
        "forward_z" => listener.forward[2] = value,
        "up_x" => listener.up[0] = value,
        "up_y" => listener.up[1] = value,
        "up_z" => listener.up[2] = value,
        "velocity_x" => listener.velocity[0] = value,
        "velocity_y" => listener.velocity[1] = value,
        "velocity_z" => listener.velocity[2] = value,
        "left_ear_offset_x" => listener.left_ear_offset[0] = value,
        "left_ear_offset_y" => listener.left_ear_offset[1] = value,
        "left_ear_offset_z" => listener.left_ear_offset[2] = value,
        "right_ear_offset_x" => listener.right_ear_offset[0] = value,
        "right_ear_offset_y" => listener.right_ear_offset[1] = value,
        "right_ear_offset_z" => listener.right_ear_offset[2] = value,
        _ => return Err(unsupported_automation_parameter("listener", parameter)),
    }
    Ok(())
}

fn apply_volume_parameter(
    volume: &mut SoundVolumeDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "priority" => volume.priority = i32_from_value(parameter, value)?,
        "interior_gain" => volume.interior_gain = value,
        "exterior_gain" => volume.exterior_gain = value,
        "low_pass_cutoff_hz" => {
            volume.low_pass_cutoff_hz = (value > 0.0).then_some(value);
        }
        "reverb_send" => volume.reverb_send = value,
        "crossfade_distance" => volume.crossfade_distance = value,
        _ => return Err(unsupported_automation_parameter("volume", parameter)),
    }
    Ok(())
}

fn ensure_finite_value(label: &str, value: f32) -> Result<(), SoundError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must be finite"
        )))
    }
}

fn bool_from_value(value: f32) -> bool {
    value >= 0.5
}

fn non_negative_usize(
    parameter: &SoundParameterId,
    value: f32,
) -> Result<usize, SoundError> {
    if value < 0.0 || value > usize::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must be a non-negative frame count",
            parameter.as_str()
        )));
    }
    Ok(value.round() as usize)
}

fn u8_from_value(parameter: &SoundParameterId, value: f32) -> Result<u8, SoundError> {
    if value < 0.0 || value > u8::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in u8",
            parameter.as_str()
        )));
    }
    Ok(value.round() as u8)
}

fn i32_from_value(parameter: &SoundParameterId, value: f32) -> Result<i32, SoundError> {
    if value < i32::MIN as f32 || value > i32::MAX as f32 {
        return Err(SoundError::InvalidParameter(format!(
            "parameter {} must fit in i32",
            parameter.as_str()
        )));
    }
    Ok(value.round() as i32)
}

fn unsupported_automation_parameter(target: &str, parameter: &SoundParameterId) -> SoundError {
    SoundError::InvalidParameter(format!(
        "unsupported sound automation parameter {} for {target}",
        parameter.as_str()
    ))
}

fn validate_source_descriptor(
    state: &SoundEngineState,
    source: &SoundSourceDescriptor,
) -> Result<(), SoundError> {
    if !source.gain.is_finite() {
        return Err(SoundError::InvalidParameter(
            "source gain must be finite".to_string(),
        ));
    }
    validate_vec3("source position", source.position)?;
    validate_vec3("source forward", source.forward)?;
    validate_vec3("source velocity", source.velocity)?;
    validate_spatial_settings(source)?;
    if let zircon_runtime::core::framework::sound::SoundSourceInput::Clip(clip) = &source.input {
        if !state.clips.contains_key(clip) {
            return Err(SoundError::UnknownClip { clip: *clip });
        }
    }
    for send in &source.sends {
        if !send.gain.is_finite() {
            return Err(SoundError::InvalidParameter(
                "source send gain must be finite".to_string(),
            ));
        }
        if !state
            .graph
            .tracks
            .iter()
            .any(|track| track.id == send.target)
        {
            return Err(SoundError::UnknownTrack { track: send.target });
        }
    }
    Ok(())
}

fn validate_listener_descriptor(
    state: &SoundEngineState,
    listener: &SoundListenerDescriptor,
) -> Result<(), SoundError> {
    validate_vec3("listener position", listener.position)?;
    validate_vec3("listener forward", listener.forward)?;
    validate_vec3("listener up", listener.up)?;
    validate_vec3("listener left ear offset", listener.left_ear_offset)?;
    validate_vec3("listener right ear offset", listener.right_ear_offset)?;
    validate_vec3("listener velocity", listener.velocity)?;
    if !state
        .graph
        .tracks
        .iter()
        .any(|track| track.id == listener.mixer_target)
    {
        return Err(SoundError::UnknownTrack {
            track: listener.mixer_target,
        });
    }
    Ok(())
}

fn validate_volume_descriptor(volume: &SoundVolumeDescriptor) -> Result<(), SoundError> {
    if !volume.interior_gain.is_finite()
        || !volume.exterior_gain.is_finite()
        || !volume.reverb_send.is_finite()
        || !volume.crossfade_distance.is_finite()
        || volume.crossfade_distance < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "volume gains, reverb send, and crossfade distance must be finite".to_string(),
        ));
    }
    if let Some(cutoff) = volume.low_pass_cutoff_hz {
        if !cutoff.is_finite() || cutoff <= 0.0 {
            return Err(SoundError::InvalidParameter(
                "volume low-pass cutoff must be positive and finite".to_string(),
            ));
        }
    }
    match &volume.shape {
        zircon_runtime::core::framework::sound::SoundVolumeShape::Sphere { center, radius } => {
            validate_vec3("volume sphere center", *center)?;
            if !radius.is_finite() || *radius < 0.0 {
                return Err(SoundError::InvalidParameter(
                    "volume sphere radius must be non-negative and finite".to_string(),
                ));
            }
        }
        zircon_runtime::core::framework::sound::SoundVolumeShape::Box { center, extents } => {
            validate_vec3("volume box center", *center)?;
            validate_vec3("volume box extents", *extents)?;
            if extents.iter().any(|extent| *extent < 0.0) {
                return Err(SoundError::InvalidParameter(
                    "volume box extents must be non-negative".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_spatial_settings(source: &SoundSourceDescriptor) -> Result<(), SoundError> {
    let spatial = source.spatial;
    if !(0.0..=1.0).contains(&spatial.spatial_blend)
        || !spatial.min_distance.is_finite()
        || !spatial.max_distance.is_finite()
        || spatial.min_distance < 0.0
        || spatial.max_distance < spatial.min_distance
        || !spatial.cone_inner_degrees.is_finite()
        || !spatial.cone_outer_degrees.is_finite()
        || spatial.cone_inner_degrees < 0.0
        || spatial.cone_outer_degrees < spatial.cone_inner_degrees
        || spatial.cone_outer_degrees > 360.0
        || !spatial.doppler_factor.is_finite()
        || spatial.doppler_factor < 0.0
    {
        return Err(SoundError::InvalidParameter(
            "source spatial settings are outside the supported range".to_string(),
        ));
    }
    Ok(())
}

fn validate_vec3(label: &str, value: [f32; 3]) -> Result<(), SoundError> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(())
    } else {
        Err(SoundError::InvalidParameter(format!(
            "{label} must contain finite coordinates"
        )))
    }
}
