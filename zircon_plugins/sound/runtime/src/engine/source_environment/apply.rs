use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundHrtfProfileDescriptor, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundSourceId, SoundVolumeDescriptor,
};

use super::super::{SoundHrtfRenderState, SoundHrtfRenderStateKey};
use super::{convolution, hrtf, spatial, volume};

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
        let spatial_scale = source
            .spatial
            .spatial_scale
            .unwrap_or(spatial_scale)
            .max(0.0);
        let active_volume = volume::strongest_volume_influence(source.position, volumes);
        let spatial = spatial::spatial_profile(
            source_id,
            source,
            listener,
            spatial_scale,
            active_volume.as_ref().map(|volume| volume.descriptor.id),
            ray_traced_impulse_responses,
        );
        if !hrtf::apply_loaded_hrtf_profile_for_source(
            buffer,
            channels,
            source_id,
            listener,
            hrtf_profiles,
            hrtf_states,
        ) {
            hrtf::apply_hrtf_preview(
                buffer,
                channels,
                source,
                listener,
                sample_rate_hz,
                source.spatial.spatial_blend.clamp(0.0, 1.0),
                spatial_scale,
            );
        }
        gain *= spatial.gain;
        pan = spatial.pan;
    }

    if let Some(influence) = volume::strongest_volume_influence(source.position, volumes) {
        gain *= influence.gain();
        if let Some(cutoff_hz) = influence.descriptor.low_pass_cutoff_hz {
            volume::low_pass_block(
                buffer,
                channels,
                sample_rate_hz,
                cutoff_hz,
                influence.weight,
            );
        }
        if let Some(impulse_response) = influence.descriptor.convolution_send {
            convolution::add_convolution_send(
                buffer,
                channels,
                impulse_responses.get(&impulse_response).map(Vec::as_slice),
                influence.descriptor.reverb_send * influence.weight,
                ray_tracing,
            );
        }
    }

    if let Some(impulse_response) = source.spatial.convolution_send {
        convolution::add_convolution_send(
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
    spatial::apply_source_pan(buffer, channels, pan);
}
