use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundRayTracingConvolutionStatus, SoundSourceDescriptor,
    SoundVolumeDescriptor,
};

use super::super::{convolution, volume};

pub(super) fn apply_volume_environment(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    source: &SoundSourceDescriptor,
    volumes: &[SoundVolumeDescriptor],
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    ray_tracing: &SoundRayTracingConvolutionStatus,
) -> f32 {
    let Some(influence) = volume::strongest_volume_influence(source.position, volumes) else {
        return 1.0;
    };

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

    influence.gain()
}
