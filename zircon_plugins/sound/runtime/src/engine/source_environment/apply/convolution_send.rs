use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundImpulseResponseId, SoundRayTracingConvolutionStatus, SoundSourceDescriptor,
};

use super::super::convolution;

pub(super) fn apply_source_convolution_send(
    buffer: &mut [f32],
    channels: usize,
    source: &SoundSourceDescriptor,
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    ray_tracing: &SoundRayTracingConvolutionStatus,
) {
    if let Some(impulse_response) = source.spatial.convolution_send {
        convolution::add_convolution_send(
            buffer,
            channels,
            impulse_responses.get(&impulse_response).map(Vec::as_slice),
            source.spatial.spatial_blend.clamp(0.0, 1.0),
            ray_tracing,
        );
    }
}
