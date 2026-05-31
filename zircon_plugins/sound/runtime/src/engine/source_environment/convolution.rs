use zircon_runtime::core::framework::sound::SoundRayTracingConvolutionStatus;

pub(super) fn add_convolution_send(
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
