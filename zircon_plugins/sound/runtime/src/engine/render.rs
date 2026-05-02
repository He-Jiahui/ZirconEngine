use std::collections::{HashMap, HashSet};

use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{
    SoundAttenuationMode, SoundError, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundMixBlock, SoundRayTracingConvolutionStatus, SoundSourceDescriptor, SoundSourceInput,
    SoundTrackId, SoundVolumeDescriptor, SoundVolumeShape,
};

use crate::SoundConfig;

use super::dsp::{apply_track_controls, apply_track_effects, meter_for};
use super::state::{ActivePlayback, SoundEngineState, SourceVoice};
use super::validation::{track_render_order, validate_graph};

const OCCLUSION_FALLBACK_GAIN: f32 = 0.7;
const SPEED_OF_SOUND_METERS_PER_SECOND: f32 = 343.0;
const MAX_DOPPLER_PREVIEW_GAIN_OFFSET: f32 = 0.1;

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
                );
            }
            apply_track_controls(&mut buffer, channels, track.controls);
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
                finished.push(*playback_id);
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
                    finished.push(*playback_id);
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
                finished.push(*playback_id);
            }
        }
        for playback_id in finished {
            self.playbacks.remove(&playback_id);
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
        let parameters = self.parameters.clone();
        let listeners = self.listeners.values().cloned().collect::<Vec<_>>();
        let volumes = self.volumes.values().cloned().collect::<Vec<_>>();
        let impulse_responses = self.impulse_responses.clone();
        let ray_tracing = self.ray_tracing.clone();
        for voice in self.sources.values_mut() {
            if !voice.descriptor.playing {
                continue;
            }
            let output_track = if track_buffers.contains_key(&voice.descriptor.output_track) {
                voice.descriptor.output_track
            } else {
                SoundTrackId::master()
            };
            let mut source_buffer = vec![0.0; frames.saturating_mul(channels)];
            mix_source_voice(
                &mut source_buffer,
                channels,
                frames,
                voice,
                &clips,
                &parameters,
                config,
            );
            if !accepts_direct_input(output_track, solo_tracks) {
                continue;
            }
            let dry_source_buffer = source_buffer.clone();
            apply_source_environment(
                &mut source_buffer,
                channels,
                config.sample_rate_hz,
                &voice.descriptor,
                active_listener_for(&listeners, output_track),
                &volumes,
                &impulse_responses,
                &ray_tracing,
            );
            if let Some(destination) = track_buffers.get_mut(&output_track) {
                add_scaled(destination, &source_buffer, 1.0);
            }
            for send in &voice.descriptor.sends {
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

fn mix_clip_playback(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    clip: &SoundAsset,
    playback: &mut ActivePlayback,
    _config: &SoundConfig,
) -> bool {
    let clip_channels = clip.channel_count as usize;
    let frame_count = clip.frame_count();
    if frame_count == 0 || clip_channels == 0 {
        return true;
    }

    for frame_index in 0..frames {
        if playback.cursor_frame >= frame_count {
            if playback.looped {
                playback.cursor_frame = 0;
            } else {
                return true;
            }
        }

        let clip_frame_offset = playback.cursor_frame * clip_channels;
        let clip_frame = &clip.samples[clip_frame_offset..clip_frame_offset + clip_channels];
        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            let mut sample = sample_for_output_channel(clip_frame, channel, output_channels);
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
            destination[output_offset + channel] += sample;
        }
        playback.cursor_frame += 1;
    }

    false
}

fn mix_source_voice(
    destination: &mut [f32],
    output_channels: usize,
    frames: usize,
    voice: &mut SourceVoice,
    clips: &HashMap<zircon_runtime::core::framework::sound::SoundClipId, super::LoadedClip>,
    parameters: &HashMap<zircon_runtime::core::framework::sound::SoundParameterId, f32>,
    config: &SoundConfig,
) {
    match &voice.descriptor.input {
        SoundSourceInput::Clip(clip_id) => {
            let Some(clip) = clips.get(clip_id) else {
                return;
            };
            let mut playback = ActivePlayback {
                clip: *clip_id,
                cursor_frame: voice.cursor_frame,
                gain: voice.descriptor.gain,
                looped: voice.descriptor.looped,
                output_track: voice.descriptor.output_track,
                pan: 0.0,
            };
            mix_clip_playback(
                destination,
                output_channels,
                frames,
                &clip.asset,
                &mut playback,
                config,
            );
            voice.cursor_frame = playback.cursor_frame;
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
                *sample += value * voice.descriptor.gain;
            }
        }
        SoundSourceInput::External(_) | SoundSourceInput::Silence => {}
    }
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

fn apply_source_environment(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    source: &SoundSourceDescriptor,
    listener: Option<&SoundListenerDescriptor>,
    volumes: &[SoundVolumeDescriptor],
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    ray_tracing: &SoundRayTracingConvolutionStatus,
) {
    let mut gain = 1.0;
    let mut pan = 0.0;

    if let Some(listener) = listener {
        let spatial = spatial_profile(source, listener);
        gain *= spatial.gain;
        pan = spatial.pan;
    }

    if let Some(volume) = strongest_volume_influence(source.position, volumes) {
        gain *= volume.gain();
        if let Some(cutoff_hz) = volume.descriptor.low_pass_cutoff_hz {
            low_pass_block(buffer, channels, sample_rate_hz, cutoff_hz, volume.weight);
        }
        if let Some(impulse_response) = volume.descriptor.convolution_send {
            add_convolution_send(
                buffer,
                channels,
                impulse_responses.get(&impulse_response).map(Vec::as_slice),
                volume.descriptor.reverb_send * volume.weight,
                ray_tracing,
            );
        }
    }

    if let Some(impulse_response) = source.spatial.convolution_send {
        add_convolution_send(
            buffer,
            channels,
            impulse_responses.get(&impulse_response).map(Vec::as_slice),
            source.spatial.spatial_blend.clamp(0.0, 1.0),
            ray_tracing,
        );
    }

    if gain != 1.0 {
        for sample in buffer.iter_mut() {
            *sample *= gain;
        }
    }
    apply_source_pan(buffer, channels, pan);
}

fn active_listener_for(
    listeners: &[SoundListenerDescriptor],
    output_track: SoundTrackId,
) -> Option<&SoundListenerDescriptor> {
    listeners
        .iter()
        .filter(|listener| listener.active)
        .min_by_key(|listener| {
            let rank = if listener.mixer_target == output_track {
                0_u8
            } else if listener.mixer_target == SoundTrackId::master() {
                1
            } else {
                2
            };
            (rank, listener.id.raw())
        })
}

#[derive(Clone, Copy, Debug)]
struct SpatialProfile {
    gain: f32,
    pan: f32,
}

fn spatial_profile(
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
) -> SpatialProfile {
    let blend = source.spatial.spatial_blend.clamp(0.0, 1.0);
    if blend <= 0.0 {
        return SpatialProfile {
            gain: 1.0,
            pan: 0.0,
        };
    }

    let offset = sub3(source.position, listener.position);
    let distance = length3(offset);
    let attenuation = attenuation_gain(
        distance,
        source.spatial.min_distance,
        source.spatial.max_distance,
        source.spatial.attenuation,
    );
    let cone = cone_gain(source.forward, source.position, listener.position, source);
    let occlusion = if source.spatial.occlusion_enabled {
        OCCLUSION_FALLBACK_GAIN
    } else {
        1.0
    };
    let doppler = doppler_preview_gain(source, listener, offset);
    let listener_right = normalize3(cross3(listener.up, listener.forward));
    let direction = normalize3(offset);

    SpatialProfile {
        gain: ((1.0 - blend) + attenuation * blend) * cone * occlusion * doppler,
        pan: dot3(direction, listener_right).clamp(-1.0, 1.0) * blend,
    }
}

fn attenuation_gain(
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    mode: SoundAttenuationMode,
) -> f32 {
    if matches!(mode, SoundAttenuationMode::None) {
        return 1.0;
    }

    let min_distance = min_distance.max(0.0001);
    let max_distance = max_distance.max(min_distance);
    if distance <= min_distance {
        return 1.0;
    }
    if distance >= max_distance {
        return 0.0;
    }

    match mode {
        SoundAttenuationMode::None => 1.0,
        SoundAttenuationMode::Linear => {
            1.0 - ((distance - min_distance) / (max_distance - min_distance).max(0.0001))
        }
        SoundAttenuationMode::InverseDistance => min_distance / distance.max(min_distance),
        SoundAttenuationMode::InverseDistanceSquared => {
            (min_distance / distance.max(min_distance)).powi(2)
        }
    }
    .clamp(0.0, 1.0)
}

fn cone_gain(
    source_forward: [f32; 3],
    source_position: [f32; 3],
    listener_position: [f32; 3],
    source: &SoundSourceDescriptor,
) -> f32 {
    let outer = source.spatial.cone_outer_degrees.clamp(0.0, 360.0);
    if outer >= 360.0 {
        return 1.0;
    }
    let inner = source.spatial.cone_inner_degrees.clamp(0.0, outer);
    let forward = normalize3(source_forward);
    let to_listener = normalize3(sub3(listener_position, source_position));
    let angle = dot3(forward, to_listener)
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();
    let inner_half = inner * 0.5;
    let outer_half = outer * 0.5;
    if angle <= inner_half {
        1.0
    } else if angle >= outer_half {
        0.0
    } else {
        1.0 - ((angle - inner_half) / (outer_half - inner_half).max(0.0001))
    }
}

fn doppler_preview_gain(
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    listener_to_source: [f32; 3],
) -> f32 {
    if !listener.doppler_tracking || source.spatial.doppler_factor <= 0.0 {
        return 1.0;
    }
    let direction_to_listener = normalize3([
        -listener_to_source[0],
        -listener_to_source[1],
        -listener_to_source[2],
    ]);
    let source_velocity = dot3(source.velocity, direction_to_listener);
    let listener_velocity = dot3(listener.velocity, direction_to_listener);
    let speed = SPEED_OF_SOUND_METERS_PER_SECOND;
    let ratio = ((speed - listener_velocity) / (speed - source_velocity).max(1.0)).clamp(0.5, 2.0);
    (1.0 + (ratio - 1.0) * source.spatial.doppler_factor * MAX_DOPPLER_PREVIEW_GAIN_OFFSET)
        .clamp(0.5, 1.5)
}

#[derive(Clone, Copy, Debug)]
struct VolumeInfluence<'a> {
    descriptor: &'a SoundVolumeDescriptor,
    weight: f32,
}

impl VolumeInfluence<'_> {
    fn gain(self) -> f32 {
        self.descriptor.exterior_gain
            + (self.descriptor.interior_gain - self.descriptor.exterior_gain) * self.weight
    }
}

fn strongest_volume_influence(
    source_position: [f32; 3],
    volumes: &[SoundVolumeDescriptor],
) -> Option<VolumeInfluence<'_>> {
    volumes
        .iter()
        .filter_map(|volume| {
            let weight = volume_weight(source_position, volume);
            (weight > 0.0).then_some(VolumeInfluence {
                descriptor: volume,
                weight,
            })
        })
        .max_by(|a, b| {
            a.descriptor
                .priority
                .cmp(&b.descriptor.priority)
                .then_with(|| b.descriptor.id.raw().cmp(&a.descriptor.id.raw()))
        })
}

fn volume_weight(source_position: [f32; 3], volume: &SoundVolumeDescriptor) -> f32 {
    let distance_outside = match &volume.shape {
        SoundVolumeShape::Sphere { center, radius } => {
            (length3(sub3(source_position, *center)) - radius.max(0.0)).max(0.0)
        }
        SoundVolumeShape::Box { center, extents } => {
            let delta = [
                (source_position[0] - center[0]).abs() - extents[0].max(0.0),
                (source_position[1] - center[1]).abs() - extents[1].max(0.0),
                (source_position[2] - center[2]).abs() - extents[2].max(0.0),
            ];
            length3([delta[0].max(0.0), delta[1].max(0.0), delta[2].max(0.0)])
        }
    };
    if distance_outside <= 0.0 {
        return 1.0;
    }
    let crossfade = volume.crossfade_distance.max(0.0);
    if crossfade <= 0.0 {
        0.0
    } else {
        (1.0 - distance_outside / crossfade).clamp(0.0, 1.0)
    }
}

fn low_pass_block(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    cutoff_hz: f32,
    amount: f32,
) {
    if cutoff_hz <= 0.0 || amount <= 0.0 {
        return;
    }
    let dry = buffer.to_vec();
    let rc = 1.0 / (cutoff_hz * std::f32::consts::TAU);
    let dt = 1.0 / sample_rate_hz.max(1) as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
    for channel in 0..channels {
        let mut low = 0.0;
        for frame in 0..(buffer.len() / channels) {
            let index = frame * channels + channel;
            low += alpha * (dry[index] - low);
            buffer[index] = dry[index] * (1.0 - amount) + low * amount;
        }
    }
}

fn add_convolution_send(
    buffer: &mut [f32],
    channels: usize,
    impulse_response: Option<&[f32]>,
    gain: f32,
    ray_tracing: &SoundRayTracingConvolutionStatus,
) {
    if gain <= 0.0 {
        return;
    }
    let Some(impulse_response) = impulse_response else {
        return;
    };
    if impulse_response.is_empty() {
        return;
    }
    let send_gain = match ray_tracing {
        SoundRayTracingConvolutionStatus::Disabled
        | SoundRayTracingConvolutionStatus::WaitingForGeometryProvider
        | SoundRayTracingConvolutionStatus::StaticImpulseResponse => gain,
        SoundRayTracingConvolutionStatus::RayTraced { .. } => gain,
    };
    let dry = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        for channel in 0..channels {
            let mut wet = 0.0;
            for (tap, coefficient) in impulse_response.iter().copied().enumerate() {
                if let Some(source_frame) = frame.checked_sub(tap) {
                    wet += dry[source_frame * channels + channel] * coefficient;
                }
            }
            buffer[frame * channels + channel] += wet * send_gain;
        }
    }
}

fn apply_source_pan(buffer: &mut [f32], channels: usize, pan: f32) {
    if channels < 2 || pan == 0.0 {
        return;
    }
    let pan = pan.clamp(-1.0, 1.0);
    let left_gain = if pan > 0.0 { 1.0 - pan } else { 1.0 };
    let right_gain = if pan < 0.0 { 1.0 + pan } else { 1.0 };
    for frame in 0..(buffer.len() / channels) {
        buffer[frame * channels] *= left_gain;
        buffer[frame * channels + 1] *= right_gain;
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length3(value: [f32; 3]) -> f32 {
    dot3(value, value).sqrt()
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = length3(value);
    if length <= 0.0001 {
        [0.0, 0.0, 1.0]
    } else {
        [value[0] / length, value[1] / length, value[2] / length]
    }
}
