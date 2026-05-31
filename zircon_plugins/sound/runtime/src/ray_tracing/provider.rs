use zircon_runtime::core::framework::sound::{
    SoundError, SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
};

use crate::engine::SoundEngineState;

use super::status::refresh_ray_tracing_status;
use super::validation::validate_ray_traced_impulse_response;

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
