use zircon_runtime::core::framework::sound::{
    SoundError, SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus,
};

use crate::ray_tracing::provider::{
    clear_ray_traced_impulse_response, submit_ray_traced_impulse_response,
};
use crate::ray_tracing::validation::validate_ray_tracing_status;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn set_ray_tracing_convolution_status_impl(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError> {
        validate_ray_tracing_status(&status)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .ray_tracing = status;
        Ok(())
    }

    pub(super) fn submit_ray_traced_impulse_response_impl(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        submit_ray_traced_impulse_response(&mut state, descriptor)
    }

    pub(super) fn ray_traced_impulse_responses_impl(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .ray_traced_impulse_responses
            .values()
            .cloned()
            .collect())
    }

    pub(super) fn clear_ray_traced_impulse_response_impl(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        clear_ray_traced_impulse_response(&mut state, impulse_response)
    }
}
