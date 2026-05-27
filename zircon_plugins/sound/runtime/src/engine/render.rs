use std::collections::{HashMap, HashSet};

use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundError, SoundExternalSourceBlock, SoundMixBlock,
    SoundParameterId, SoundPlaybackCompletionAction, SoundPlaybackFinishReason,
    SoundPlaybackFinished, SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceFinished,
    SoundSourceInput, SoundTrackId,
};

use crate::SoundConfig;

use super::dsp::{apply_track_controls, apply_track_effects, meter_for};
use super::hrtf::prune_hrtf_render_states;
use super::source_environment::{
    active_listener_for, apply_source_environment, hrtf_tail_pending_for_source,
};
use super::state::{ActivePlayback, SoundEngineState, SourceVoice};
use super::validation::{track_render_order, validate_graph};
use super::{SoundEffectStateKey, SoundHrtfRenderStateKey};

impl SoundEngineState {
    pub(crate) fn render_mix(
        &mut self,
        config: &SoundConfig,
        frames: usize,
    ) -> Result<SoundMixBlock, SoundError> {
        if frames == 0 {
            return Err(SoundError::InvalidMixRequest { frames });
        }
        validate_graph(&self.graph)?;
        sync_runtime_states(self);
        self.latency_frames = latency_frames_for_graph(&self.graph);

        let channels = config.channel_count.max(1) as usize;
        let samples_len = frames.saturating_mul(channels);
        let mut track_buffers = self
            .graph
            .tracks
            .iter()
            .map(|track| (track.id, vec![0.0; samples_len]))
            .collect::<HashMap<_, _>>();
        let solo_tracks = solo_tracks(&self.graph);

        self.mix_playbacks(config, frames, channels, &solo_tracks, &mut track_buffers);
        self.mix_sources(config, frames, channels, &solo_tracks, &mut track_buffers);
        let mut pre_effect_sidechain_buffers = track_buffers.clone();
        let mut post_effect_sidechain_buffers = HashMap::new();

        let mut meters = Vec::new();
        let tracks = self.graph.tracks.clone();
        for track_id in track_render_order(&self.graph) {
            let Some(track) = tracks.iter().find(|track| track.id == track_id) else {
                continue;
            };
            let Some(mut buffer) = track_buffers.remove(&track_id) else {
                continue;
            };
            let raw_buffer = buffer.clone();
            pre_effect_sidechain_buffers.insert(track_id, raw_buffer.clone());
            if !track.controls.bypass_effects {
                apply_track_effects(
                    &mut buffer,
                    channels,
                    config.sample_rate_hz,
                    &track.effects,
                    &pre_effect_sidechain_buffers,
                    &post_effect_sidechain_buffers,
                    &self.impulse_responses,
                    track_id,
                    &mut self.effect_states,
                );
            }
            let track_state = self.track_states.entry(track_id).or_default();
            apply_track_controls(&mut buffer, channels, track.controls, track_state);
            post_effect_sidechain_buffers.insert(track_id, buffer.clone());
            meters.push(meter_for(track_id, &buffer, channels));

            if track_id == SoundTrackId::master() {
                track_buffers.insert(track_id, buffer);
                continue;
            }

            if let Some(parent) = track.parent {
                if let Some(parent_buffer) = track_buffers.get_mut(&parent) {
                    add_scaled(parent_buffer, &buffer, 1.0);
                }
            }
            for send in &track.sends {
                if let Some(send_buffer) = track_buffers.get_mut(&send.target) {
                    let source = if send.pre_effects {
                        &raw_buffer
                    } else {
                        &buffer
                    };
                    add_scaled(send_buffer, source, send.gain);
                }
            }
        }

        self.meters = meters;
        let mut mix = SoundMixBlock {
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count.max(1),
            samples: track_buffers
                .remove(&SoundTrackId::master())
                .unwrap_or_else(|| vec![0.0; samples_len]),
        };
        for sample in &mut mix.samples {
            *sample = (*sample * config.master_gain).clamp(-1.0, 1.0);
        }
        Ok(mix)
    }

    fn mix_playbacks(
        &mut self,
        config: &SoundConfig,
        frames: usize,
        channels: usize,
        solo_tracks: &HashSet<SoundTrackId>,
        track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    ) {
        let clips = self.clips.clone();
        let mut finished = Vec::new();
        for (playback_id, playback) in self.playbacks.iter_mut() {
            let Some(clip) = clips.get(&playback.clip) else {
                finished.push((*playback_id, SoundPlaybackFinishReason::MissingClip));
                continue;
            };
            let output_track = if track_buffers.contains_key(&playback.output_track) {
                playback.output_track
            } else {
                SoundTrackId::master()
            };
            if !accepts_direct_input(output_track, solo_tracks) {
                let mut scratch = vec![0.0; frames.saturating_mul(channels)];
                if mix_clip_playback(
                    &mut scratch,
                    channels,
                    frames,
                    &clip.asset,
                    playback,
                    config,
                ) {
                    finished.push((*playback_id, SoundPlaybackFinishReason::Completed));
                }
                continue;
            }
            let finished_playback = if let Some(destination) = track_buffers.get_mut(&output_track)
            {
                mix_clip_playback(destination, channels, frames, &clip.asset, playback, config)
            } else {
                false
            };
            if finished_playback {
                finished.push((*playback_id, SoundPlaybackFinishReason::Completed));
            }
        }
        for (playback_id, reason) in finished {
            if let Some(active) = self.playbacks.remove(&playback_id) {
                self.finished_playbacks.push(SoundPlaybackFinished {
                    playback: playback_id,
                    clip: active.clip,
                    reason,
                    completion_action: active.completion_action,
                    output_track: active.output_track,
                });
            }
        }
    }

    fn mix_sources(
        &mut self,
        config: &SoundConfig,
        frames: usize,
        channels: usize,
        solo_tracks: &HashSet<SoundTrackId>,
        track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    ) {
        let clips = self.clips.clone();
        let external_sources = self.external_sources.clone();
        let parameters = self.parameters.clone();
        let listeners = self.listeners.values().cloned().collect::<Vec<_>>();
        let volumes = self.volumes.values().cloned().collect::<Vec<_>>();
        let impulse_responses = self.impulse_responses.clone();
        let ray_traced_impulse_responses = self.ray_traced_impulse_responses.clone();
        let hrtf_profiles = self.hrtf_profiles.clone();
        let ray_tracing = self.ray_tracing.clone();
        let mut finished = Vec::new();
        for (source_id, voice) in self.sources.iter_mut() {
            let source_descriptor =
                source_descriptor_with_parameter_bindings(&voice.descriptor, &parameters);
            if !source_descriptor.playing {
                continue;
            }
            let output_track = if track_buffers.contains_key(&source_descriptor.output_track) {
                source_descriptor.output_track
            } else {
                SoundTrackId::master()
            };
            let mut source_buffer = vec![0.0; frames.saturating_mul(channels)];
            if voice.pending_finish.is_none() {
                voice.pending_finish = mix_source_voice(
                    &mut source_buffer,
                    channels,
                    frames,
                    voice,
                    &source_descriptor,
                    &clips,
                    &external_sources,
                    &parameters,
                    config,
                );
            }
            if !accepts_direct_input(output_track, solo_tracks) {
                continue;
            }
            let dry_source_buffer = source_buffer.clone();
            apply_source_environment(
                &mut source_buffer,
                channels,
                config.sample_rate_hz,
                *source_id,
                &source_descriptor,
                active_listener_for(&listeners, output_track),
                config.default_spatial_scale,
                &volumes,
                &impulse_responses,
                &ray_traced_impulse_responses,
                &hrtf_profiles,
                &mut self.hrtf_states,
                &ray_tracing,
            );
            if let Some(reason) = voice.pending_finish {
                let has_tail = hrtf_tail_pending_for_source(
                    &self.hrtf_states,
                    *source_id,
                    active_listener_for(&listeners, output_track),
                    &hrtf_profiles,
                );
                if !has_tail {
                    finished.push((*source_id, source_descriptor.clone(), reason));
                }
            }
            if let Some(destination) = track_buffers.get_mut(&output_track) {
                add_scaled(destination, &source_buffer, 1.0);
            }
            for send in &source_descriptor.sends {
                if let Some(destination) = track_buffers.get_mut(&send.target) {
                    let source = if send.pre_spatial {
                        &dry_source_buffer
                    } else {
                        &source_buffer
                    };
                    add_scaled(destination, source, send.gain);
                }
            }
        }
        for (source_id, descriptor, reason) in finished {
            if self.sources.remove(&source_id).is_some() {
                let input = descriptor.input;
                let clip = match input {
                    SoundSourceInput::Clip(clip) => Some(clip),
                    SoundSourceInput::External(_)
                    | SoundSourceInput::SynthParameter { .. }
                    | SoundSourceInput::Silence => None,
                };
                self.finished_sources.push(SoundSourceFinished {
                    source: source_id,
                    input,
                    clip,
                    reason,
                    completion_action: descriptor.completion_action,
                    output_track: descriptor.output_track,
                });
            }
        }
    }
}

fn solo_tracks(
    graph: &zircon_runtime::core::framework::sound::SoundMixerGraph,
) -> HashSet<SoundTrackId> {
    graph
        .tracks
        .iter()
        .filter(|track| track.controls.solo)
        .map(|track| track.id)
        .collect()
}

fn accepts_direct_input(track: SoundTrackId, solo_tracks: &HashSet<SoundTrackId>) -> bool {
    solo_tracks.is_empty() || solo_tracks.contains(&track)
}

fn sync_runtime_states(state: &mut SoundEngineState) {
    let track_ids = state
        .graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    let effect_keys = state
        .graph
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .effects
                .iter()
                .map(move |effect| SoundEffectStateKey::new(track.id, effect.id))
        })
        .collect::<HashSet<_>>();

    state
        .track_states
        .retain(|track, _| track_ids.contains(track));
    for track in &track_ids {
        state.track_states.entry(*track).or_default();
    }
    state
        .effect_states
        .retain(|effect, _| effect_keys.contains(effect));

    let listeners = state.listeners.values().cloned().collect::<Vec<_>>();
    let parameters = state.parameters.clone();
    let active_hrtf_keys = state
        .sources
        .iter()
        .filter_map(|(source_id, voice)| {
            let descriptor =
                source_descriptor_with_parameter_bindings(&voice.descriptor, &parameters);
            if !descriptor.playing && voice.pending_finish.is_none() {
                return None;
            }
            let output_track = if track_ids.contains(&descriptor.output_track) {
                descriptor.output_track
            } else {
                SoundTrackId::master()
            };
            let listener = active_listener_for(&listeners, output_track)?;
            let profile_id = listener.hrtf_profile.as_deref()?;
            state.hrtf_profiles.contains_key(profile_id).then(|| {
                SoundHrtfRenderStateKey::new(*source_id, listener.id, profile_id.to_string())
            })
        })
        .collect::<HashSet<_>>();
    prune_hrtf_render_states(&mut state.hrtf_states, &active_hrtf_keys);
}

fn latency_frames_for_graph(
    graph: &zircon_runtime::core::framework::sound::SoundMixerGraph,
) -> usize {
    graph
        .tracks
        .iter()
        .map(|track| {
            let effect_latency = track
                .effects
                .iter()
                .filter(|effect| effect.enabled && !effect.bypass)
                .map(effect_latency_frames)
                .max()
                .unwrap_or_default();
            track.controls.delay_frames.max(effect_latency)
        })
        .max()
        .unwrap_or_default()
}

fn effect_latency_frames(
    effect: &zircon_runtime::core::framework::sound::SoundEffectDescriptor,
) -> usize {
    match &effect.kind {
        zircon_runtime::core::framework::sound::SoundEffectKind::Delay(delay) => delay.delay_frames,
        zircon_runtime::core::framework::sound::SoundEffectKind::Reverb(reverb) => {
            reverb.pre_delay_frames.max(reverb.tail_frames)
        }
        zircon_runtime::core::framework::sound::SoundEffectKind::ConvolutionReverb(convolution) => {
            convolution.latency_frames
        }
        zircon_runtime::core::framework::sound::SoundEffectKind::Flanger(flanger) => {
            flanger.delay_frames + flanger.depth_frames
        }
        zircon_runtime::core::framework::sound::SoundEffectKind::Chorus(chorus) => {
            chorus.delay_frames + chorus.depth_frames.saturating_mul(chorus.voices as usize)
        }
        _ => 0,
    }
}

fn mix_clip_playback(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    clip: &SoundAsset,
    playback: &mut ActivePlayback,
    config: &SoundConfig,
) -> bool {
    if playback.paused {
        return false;
    }
    let clip_channels = clip.channel_count as usize;
    let frame_count = clip.frame_count();
    if frame_count == 0 || clip_channels == 0 {
        return true;
    }
    let step = resample_step(clip.sample_rate_hz, config.sample_rate_hz) * playback.speed as f64;

    for frame_index in 0..frames {
        let Some(source_frame_position) = next_clip_source_frame_position(
            &mut playback.cursor_position,
            frame_count,
            playback.range_start_frame,
            playback.range_end_frame,
            step,
            playback.looped,
        ) else {
            playback.cursor_frame = playback.range_end_frame.unwrap_or(frame_count);
            return true;
        };

        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            let mut sample = interpolated_source_sample(
                &clip.samples,
                clip_channels,
                frame_count,
                playback.range_start_frame,
                playback.range_end_frame,
                source_frame_position,
                channel,
                output_channels,
                playback.looped,
            );
            sample *= playback.gain;
            if output_channels > 1 {
                sample *= if channel == 0 {
                    if playback.pan > 0.0 {
                        1.0 - playback.pan.clamp(0.0, 1.0)
                    } else {
                        1.0
                    }
                } else if playback.pan < 0.0 {
                    1.0 + playback.pan.clamp(-1.0, 0.0)
                } else {
                    1.0
                };
            }
            if !playback.muted {
                destination[output_offset + channel] += sample;
            }
        }
        playback.cursor_frame = playback.cursor_position.floor() as usize;
    }

    false
}

fn mix_source_voice(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    voice: &mut SourceVoice,
    descriptor: &SoundSourceDescriptor,
    clips: &HashMap<zircon_runtime::core::framework::sound::SoundClipId, super::LoadedClip>,
    external_sources: &HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    parameters: &HashMap<zircon_runtime::core::framework::sound::SoundParameterId, f32>,
    config: &SoundConfig,
) -> Option<SoundSourceFinishReason> {
    match &descriptor.input {
        SoundSourceInput::Clip(clip_id) => {
            let Some(clip) = clips.get(clip_id) else {
                return Some(SoundSourceFinishReason::MissingClip);
            };
            let range = source_clip_range(
                descriptor,
                clip.asset.sample_rate_hz,
                clip.asset.frame_count(),
            );
            let mut playback = ActivePlayback {
                clip: *clip_id,
                cursor_frame: voice.cursor_frame,
                cursor_position: voice.cursor_position,
                gain: descriptor.gain,
                speed: descriptor.speed,
                looped: descriptor.looped,
                completion_action: SoundPlaybackCompletionAction::None,
                paused: false,
                muted: descriptor.muted,
                range_start_frame: range.0,
                range_end_frame: range.1,
                output_track: descriptor.output_track,
                pan: 0.0,
            };
            let finished = mix_clip_playback(
                destination,
                output_channels,
                frames,
                &clip.asset,
                &mut playback,
                config,
            );
            voice.cursor_frame = playback.cursor_frame;
            voice.cursor_position = playback.cursor_position;
            finished.then_some(SoundSourceFinishReason::Completed)
        }
        SoundSourceInput::External(handle) => {
            let Some(block) = external_sources.get(handle) else {
                return None;
            };
            let finished = mix_external_source_block(
                destination,
                output_channels,
                frames,
                block,
                descriptor.gain,
                descriptor.looped,
                config.sample_rate_hz,
                &mut voice.cursor_frame,
                &mut voice.cursor_position,
            );
            finished.then_some(SoundSourceFinishReason::Completed)
        }
        SoundSourceInput::SynthParameter {
            parameter,
            default_value,
        } => {
            let value = parameters
                .get(parameter)
                .copied()
                .unwrap_or(*default_value)
                .clamp(-1.0, 1.0);
            for sample in destination {
                *sample += value * descriptor.gain;
            }
            None
        }
        SoundSourceInput::Silence => None,
    }
}

fn source_clip_range(
    descriptor: &SoundSourceDescriptor,
    sample_rate_hz: u32,
    frame_count: usize,
) -> (usize, Option<usize>) {
    let sample_rate = sample_rate_hz.max(1) as f32;
    let start_frame = descriptor
        .start_seconds
        .map(|seconds| (seconds * sample_rate).round().max(0.0) as usize)
        .unwrap_or_default()
        .min(frame_count);
    let end_frame = descriptor.duration_seconds.map(|seconds| {
        let duration_frames = (seconds * sample_rate).round().max(0.0) as usize;
        start_frame.saturating_add(duration_frames).min(frame_count)
    });
    (start_frame, end_frame)
}

fn source_descriptor_with_parameter_bindings(
    descriptor: &SoundSourceDescriptor,
    parameters: &HashMap<SoundParameterId, f32>,
) -> SoundSourceDescriptor {
    let mut resolved = descriptor.clone();
    for binding in &descriptor.parameter_bindings {
        let Some(value) = parameters.get(&binding.synth_parameter).copied() else {
            continue;
        };
        apply_source_bound_parameter(&mut resolved, binding.source_parameter.as_str(), value);
    }
    resolved
}

fn apply_source_bound_parameter(source: &mut SoundSourceDescriptor, parameter: &str, value: f32) {
    match parameter {
        "gain" => source.gain = value,
        "speed" => source.speed = value,
        "playing" => source.playing = bool_from_parameter(value),
        "looped" => source.looped = bool_from_parameter(value),
        "muted" => source.muted = bool_from_parameter(value),
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
        "spatial_scale" => source.spatial.spatial_scale = Some(value),
        "min_distance" => source.spatial.min_distance = value,
        "max_distance" => source.spatial.max_distance = value,
        "cone_inner_degrees" => source.spatial.cone_inner_degrees = value,
        "cone_outer_degrees" => source.spatial.cone_outer_degrees = value,
        "doppler_factor" => source.spatial.doppler_factor = value,
        "occlusion_enabled" => source.spatial.occlusion_enabled = bool_from_parameter(value),
        _ => {}
    }
}

fn bool_from_parameter(value: f32) -> bool {
    value >= 0.5
}

fn mix_external_source_block(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    block: &SoundExternalSourceBlock,
    gain: f32,
    looped: bool,
    output_sample_rate_hz: u32,
    cursor_frame: &mut usize,
    cursor_position: &mut f64,
) -> bool {
    let source_channels = block.channel_count as usize;
    if source_channels == 0 {
        return !looped;
    }
    let frame_count = block.samples.len() / source_channels;
    if frame_count == 0 {
        return !looped;
    }
    let step = resample_step(block.sample_rate_hz, output_sample_rate_hz);

    for frame_index in 0..frames {
        let Some(source_frame_position) =
            next_source_frame_position(cursor_position, frame_count, step, looped)
        else {
            *cursor_frame = frame_count;
            return true;
        };

        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            destination[output_offset + channel] += interpolated_source_sample(
                &block.samples,
                source_channels,
                frame_count,
                0,
                None,
                source_frame_position,
                channel,
                output_channels,
                looped,
            ) * gain;
        }
        *cursor_frame = cursor_position.floor() as usize;
    }
    false
}

fn resample_step(source_sample_rate_hz: u32, output_sample_rate_hz: u32) -> f64 {
    source_sample_rate_hz.max(1) as f64 / output_sample_rate_hz.max(1) as f64
}

fn next_source_frame_position(
    cursor_position: &mut f64,
    frame_count: usize,
    step: f64,
    looped: bool,
) -> Option<f64> {
    if frame_count == 0 {
        return None;
    }
    if *cursor_position >= frame_count as f64 {
        if looped {
            *cursor_position %= frame_count as f64;
        } else {
            return None;
        }
    }
    let frame_position = *cursor_position;
    *cursor_position += step;
    Some(frame_position)
}

fn next_clip_source_frame_position(
    cursor_position: &mut f64,
    frame_count: usize,
    range_start_frame: usize,
    range_end_frame: Option<usize>,
    step: f64,
    looped: bool,
) -> Option<f64> {
    if frame_count == 0 {
        return None;
    }
    let start = range_start_frame.min(frame_count);
    let end = range_end_frame
        .unwrap_or(frame_count)
        .min(frame_count)
        .max(start);
    if start >= end {
        return None;
    }
    if *cursor_position < start as f64 {
        *cursor_position = start as f64;
    }
    if *cursor_position >= end as f64 {
        if looped {
            *cursor_position = start as f64;
        } else {
            return None;
        }
    }
    let frame_position = *cursor_position;
    *cursor_position += step;
    Some(frame_position)
}

fn interpolated_source_sample(
    samples: &[f32],
    source_channels: usize,
    frame_count: usize,
    range_start_frame: usize,
    range_end_frame: Option<usize>,
    frame_position: f64,
    output_channel: usize,
    output_channel_count: usize,
    looped: bool,
) -> f32 {
    if source_channels == 0 || frame_count == 0 {
        return 0.0;
    }

    let base_position = frame_position.floor().max(0.0);
    let base_frame = (base_position as usize).min(frame_count - 1);
    let range_start = range_start_frame.min(frame_count);
    let range_end = range_end_frame
        .unwrap_or(frame_count)
        .min(frame_count)
        .max(range_start);
    let next_frame = if base_frame + 1 < range_end {
        base_frame + 1
    } else if looped {
        range_start
    } else {
        base_frame
    };
    let blend = (frame_position - base_position).clamp(0.0, 1.0) as f32;
    let start = source_frame_sample(
        samples,
        source_channels,
        base_frame,
        output_channel,
        output_channel_count,
    );
    let end = source_frame_sample(
        samples,
        source_channels,
        next_frame,
        output_channel,
        output_channel_count,
    );
    start + (end - start) * blend
}

fn source_frame_sample(
    samples: &[f32],
    source_channels: usize,
    frame_index: usize,
    output_channel: usize,
    output_channel_count: usize,
) -> f32 {
    let source_frame_offset = frame_index.saturating_mul(source_channels);
    let source_frame_end = source_frame_offset.saturating_add(source_channels);
    let Some(source_frame) = samples.get(source_frame_offset..source_frame_end) else {
        return 0.0;
    };
    sample_for_output_channel(source_frame, output_channel, output_channel_count)
}

fn sample_for_output_channel(
    clip_frame: &[f32],
    output_channel: usize,
    output_channel_count: usize,
) -> f32 {
    if clip_frame.len() == 1 {
        return clip_frame[0];
    }
    if output_channel_count == 1 {
        return clip_frame.iter().copied().sum::<f32>() / clip_frame.len() as f32;
    }

    clip_frame
        .get(output_channel)
        .copied()
        .unwrap_or_else(|| *clip_frame.last().unwrap_or(&0.0))
}

fn add_scaled(destination: &mut [f32], source: &[f32], gain: f32) {
    for (destination, source) in destination.iter_mut().zip(source.iter().copied()) {
        *destination += source * gain;
    }
}
