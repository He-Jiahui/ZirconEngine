use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus,
};

use crate::engine::SoundEngineState;

pub(crate) fn submit_ray_traced_impulse_response(
    state: &mut SoundEngineState,
    descriptor: SoundRayTracedImpulseResponseDescriptor,
) -> Result<(), SoundError> {
    validate_ray_traced_impulse_response(state, &descriptor)?;
    state
        .impulse_responses
        .insert(descriptor.impulse_response, descriptor.samples.clone());
    state
        .ray_traced_impulse_responses
        .insert(descriptor.impulse_response, descriptor);
    refresh_ray_tracing_status(&mut state.ray_tracing, &state.ray_traced_impulse_responses);
    Ok(())
}

pub(crate) fn clear_ray_traced_impulse_response(
    state: &mut SoundEngineState,
    impulse_response: SoundImpulseResponseId,
) -> Result<(), SoundError> {
    state
        .ray_traced_impulse_responses
        .remove(&impulse_response)
        .ok_or(SoundError::UnknownImpulseResponse { impulse_response })?;
    state.impulse_responses.remove(&impulse_response);
    refresh_ray_tracing_status(&mut state.ray_tracing, &state.ray_traced_impulse_responses);
    Ok(())
}

pub(crate) fn refresh_ray_tracing_status(
    status: &mut SoundRayTracingConvolutionStatus,
    descriptors: &HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
) {
    if descriptors.is_empty() {
        if matches!(status, SoundRayTracingConvolutionStatus::RayTraced { .. }) {
            *status = SoundRayTracingConvolutionStatus::WaitingForGeometryProvider;
        }
        return;
    }

    let rays_per_update = descriptors
        .values()
        .map(|descriptor| descriptor.rays_traced)
        .max()
        .unwrap_or(1);
    *status = SoundRayTracingConvolutionStatus::RayTraced {
        cached_cells: descriptors.len(),
        rays_per_update,
    };
}

pub(crate) fn validate_ray_tracing_status(
    status: &SoundRayTracingConvolutionStatus,
) -> Result<(), SoundError> {
    if let SoundRayTracingConvolutionStatus::RayTraced {
        rays_per_update, ..
    } = status
    {
        if *rays_per_update == 0 {
            return Err(SoundError::InvalidParameter(
                "ray-traced convolution requires at least one ray per update".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_ray_traced_impulse_response(
    state: &SoundEngineState,
    descriptor: &SoundRayTracedImpulseResponseDescriptor,
) -> Result<(), SoundError> {
    if descriptor.cell_key.trim().is_empty()
        || descriptor.sample_rate_hz == 0
        || descriptor.channel_count == 0
        || descriptor.rays_traced == 0
        || descriptor.samples.is_empty()
        || descriptor.samples.iter().any(|sample| !sample.is_finite())
    {
        return Err(SoundError::InvalidParameter(
            "ray-traced impulse response requires a cell key, format, rays, and finite samples"
                .to_string(),
        ));
    }
    if let Some(source_id) = descriptor.source {
        if !state.sources.contains_key(&source_id) {
            return Err(SoundError::UnknownSource { source_id });
        }
    }
    if let Some(listener) = descriptor.listener {
        if !state.listeners.contains_key(&listener) {
            return Err(SoundError::UnknownListener { listener });
        }
    }
    if let Some(volume) = descriptor.volume {
        if !state.volumes.contains_key(&volume) {
            return Err(SoundError::UnknownVolume { volume });
        }
    }
    Ok(())
}
