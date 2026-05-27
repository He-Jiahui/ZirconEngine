use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundAttenuationMode, SoundHrtfProfileDescriptor, SoundImpulseResponseId,
    SoundListenerDescriptor, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus, SoundSourceDescriptor, SoundSourceId, SoundTrackId,
    SoundVolumeDescriptor, SoundVolumeId, SoundVolumeShape,
};

use super::hrtf::apply_loaded_hrtf_profile;
use super::math::{add3, cross3, dot3, length3, normalize3, scale3, sub3};
use super::{
    occlusion_gain_for_query, SoundHrtfRenderState, SoundHrtfRenderStateKey, SoundOcclusionQuery,
};

const SPEED_OF_SOUND_METERS_PER_SECOND: f32 = 343.0;
const MAX_DOPPLER_PREVIEW_GAIN_OFFSET: f32 = 0.1;
const HRTF_PREVIEW_GAIN_MIN: f32 = 0.5;
const HRTF_PREVIEW_GAIN_MAX: f32 = 1.5;
const HRTF_PREVIEW_MAX_DELAY_FRAMES: usize = 64;

pub(crate) fn apply_source_environment(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    source_id: SoundSourceId,
    source: &SoundSourceDescriptor,
    listener: Option<&SoundListenerDescriptor>,
    spatial_scale: f32,
    volumes: &[SoundVolumeDescriptor],
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
    hrtf_states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    ray_tracing: &SoundRayTracingConvolutionStatus,
) {
    let mut gain = 1.0;
    let mut pan = 0.0;

    if let Some(listener) = listener {
        let spatial_scale = source.spatial.spatial_scale.unwrap_or(spatial_scale);
        let active_volume = strongest_volume_influence(source.position, volumes);
        let spatial = spatial_profile(
            source_id,
            source,
            listener,
            sample_rate_hz,
            spatial_scale,
            active_volume.as_ref().map(|volume| volume.descriptor.id),
            ray_traced_impulse_responses,
        );
        if !apply_loaded_hrtf_profile_for_source(
            buffer,
            channels,
            source_id,
            listener,
            hrtf_profiles,
            hrtf_states,
        ) {
            apply_hrtf_preview(buffer, channels, spatial);
        }
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

pub(crate) fn active_listener_for(
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
    left_ear_gain: f32,
    right_ear_gain: f32,
    left_delay_frames: usize,
    right_delay_frames: usize,
}

fn spatial_profile(
    source_id: SoundSourceId,
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    sample_rate_hz: u32,
    spatial_scale: f32,
    volume: Option<SoundVolumeId>,
    ray_traced_impulse_responses: &HashMap<
        SoundImpulseResponseId,
        SoundRayTracedImpulseResponseDescriptor,
    >,
) -> SpatialProfile {
    let blend = source.spatial.spatial_blend.clamp(0.0, 1.0);
    if blend <= 0.0 {
        return SpatialProfile {
            gain: 1.0,
            pan: 0.0,
            left_ear_gain: 1.0,
            right_ear_gain: 1.0,
            left_delay_frames: 0,
            right_delay_frames: 0,
        };
    }

    let spatial_scale = spatial_scale.max(0.0);
    let offset = scale3(sub3(source.position, listener.position), spatial_scale);
    let distance = length3(offset);
    let attenuation = attenuation_gain(
        distance,
        source.spatial.min_distance,
        source.spatial.max_distance,
        source.spatial.attenuation,
    );
    let cone = cone_gain(source.forward, source.position, listener.position, source);
    let occlusion = occlusion_gain_for_query(
        source.spatial.occlusion_enabled,
        SoundOcclusionQuery {
            source: source.id.unwrap_or(source_id),
            listener: Some(listener.id),
            volume,
        },
        ray_traced_impulse_responses,
    );
    let doppler = doppler_preview_gain(source, listener, offset);
    let listener_right = normalize3(cross3(listener.up, listener.forward));
    let direction = normalize3(offset);
    let hrtf = hrtf_preview_profile(source, listener, sample_rate_hz, blend, spatial_scale);

    SpatialProfile {
        gain: ((1.0 - blend) + attenuation * blend) * cone * occlusion * doppler,
        pan: dot3(direction, listener_right).clamp(-1.0, 1.0) * blend,
        left_ear_gain: hrtf.left_gain,
        right_ear_gain: hrtf.right_gain,
        left_delay_frames: hrtf.left_delay_frames,
        right_delay_frames: hrtf.right_delay_frames,
    }
}

#[derive(Clone, Copy, Debug)]
struct HrtfPreviewProfile {
    left_gain: f32,
    right_gain: f32,
    left_delay_frames: usize,
    right_delay_frames: usize,
}

fn hrtf_preview_profile(
    source: &SoundSourceDescriptor,
    listener: &SoundListenerDescriptor,
    sample_rate_hz: u32,
    blend: f32,
    spatial_scale: f32,
) -> HrtfPreviewProfile {
    if listener
        .hrtf_profile
        .as_deref()
        .unwrap_or_default()
        .is_empty()
    {
        return HrtfPreviewProfile {
            left_gain: 1.0,
            right_gain: 1.0,
            left_delay_frames: 0,
            right_delay_frames: 0,
        };
    }

    let source_position = scale3(source.position, spatial_scale);
    let listener_position = scale3(listener.position, spatial_scale);
    let left_ear = add3(
        listener_position,
        scale3(listener.left_ear_offset, spatial_scale),
    );
    let right_ear = add3(
        listener_position,
        scale3(listener.right_ear_offset, spatial_scale),
    );
    let left_distance = length3(sub3(source_position, left_ear)).max(0.0001);
    let right_distance = length3(sub3(source_position, right_ear)).max(0.0001);
    let average_distance = (left_distance + right_distance) * 0.5;
    let left_gain =
        (average_distance / left_distance).clamp(HRTF_PREVIEW_GAIN_MIN, HRTF_PREVIEW_GAIN_MAX);
    let right_gain =
        (average_distance / right_distance).clamp(HRTF_PREVIEW_GAIN_MIN, HRTF_PREVIEW_GAIN_MAX);
    let delay_frames = (((left_distance - right_distance).abs() / SPEED_OF_SOUND_METERS_PER_SECOND)
        * sample_rate_hz.max(1) as f32)
        .round() as usize;
    let delay_frames = delay_frames.min(HRTF_PREVIEW_MAX_DELAY_FRAMES);
    let (left_delay_frames, right_delay_frames) = if left_distance > right_distance {
        (delay_frames, 0)
    } else {
        (0, delay_frames)
    };

    HrtfPreviewProfile {
        left_gain: 1.0 + (left_gain - 1.0) * blend,
        right_gain: 1.0 + (right_gain - 1.0) * blend,
        left_delay_frames,
        right_delay_frames,
    }
}

fn apply_hrtf_preview(buffer: &mut [f32], channels: usize, spatial: SpatialProfile) {
    if channels < 2 {
        return;
    }
    if spatial.left_ear_gain == 1.0
        && spatial.right_ear_gain == 1.0
        && spatial.left_delay_frames == 0
        && spatial.right_delay_frames == 0
    {
        return;
    }

    let dry = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let left = frame
            .checked_sub(spatial.left_delay_frames)
            .and_then(|source_frame| dry.get(source_frame * channels))
            .copied()
            .unwrap_or_default();
        let right = frame
            .checked_sub(spatial.right_delay_frames)
            .and_then(|source_frame| dry.get(source_frame * channels + 1))
            .copied()
            .unwrap_or_default();
        buffer[frame * channels] = left * spatial.left_ear_gain;
        buffer[frame * channels + 1] = right * spatial.right_ear_gain;
    }
}

fn apply_loaded_hrtf_profile_for_source(
    buffer: &mut [f32],
    channels: usize,
    source_id: SoundSourceId,
    listener: &SoundListenerDescriptor,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
    hrtf_states: &mut HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
) -> bool {
    let Some(profile_id) = listener
        .hrtf_profile
        .as_deref()
        .filter(|profile_id| !profile_id.is_empty())
    else {
        return false;
    };
    let Some(profile) = hrtf_profiles.get(profile_id) else {
        return false;
    };

    let key = SoundHrtfRenderStateKey::new(source_id, listener.id, profile_id.to_string());
    let state = hrtf_states.entry(key).or_default();
    apply_loaded_hrtf_profile(buffer, channels, profile, state)
}

pub(crate) fn hrtf_tail_pending_for_source(
    hrtf_states: &HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    source_id: SoundSourceId,
    listener: Option<&SoundListenerDescriptor>,
    hrtf_profiles: &HashMap<String, SoundHrtfProfileDescriptor>,
) -> bool {
    let Some(listener) = listener else {
        return false;
    };
    let Some(profile_id) = listener.hrtf_profile.as_deref() else {
        return false;
    };
    if !hrtf_profiles.contains_key(profile_id) {
        return false;
    }
    let key = SoundHrtfRenderStateKey::new(source_id, listener.id, profile_id.to_string());
    hrtf_states
        .get(&key)
        .is_some_and(SoundHrtfRenderState::has_pending_tail)
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
