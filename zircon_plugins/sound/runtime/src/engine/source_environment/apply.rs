use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceId, SoundVolumeDescriptor,
};

use super::super::{SoundHrtfRenderState, SoundHrtfRenderStateKey};

mod convolution_send;
mod final_mix;
mod listener;
mod volume_influence;

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
        let listener_projection = listener::apply_listener_environment(
            buffer,
            channels,
            sample_rate_hz,
            source_id,
            source,
            listener,
            spatial_scale,
            volumes,
            ray_traced_impulse_responses,
            hrtf_profiles,
            hrtf_states,
        );
        gain *= listener_projection.gain;
        pan = listener_projection.pan;
    }

    gain *= volume_influence::apply_volume_environment(
        buffer,
        channels,
        sample_rate_hz,
        source,
        volumes,
        impulse_responses,
        ray_tracing,
    );
    convolution_send::apply_source_convolution_send(
        buffer,
        channels,
        source,
        impulse_responses,
        ray_tracing,
    );
    final_mix::apply_final_gain_and_pan(buffer, channels, gain, pan);
}
