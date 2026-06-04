use zircon_runtime::core::framework::sound::{
    SoundAcousticsManager, SoundError, SoundHrtfProfileDescriptor, SoundImpulseResponseId,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
};

use super::super::DefaultSoundManager;

impl SoundAcousticsManager for DefaultSoundManager {
    fn set_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError> {
        self.set_impulse_response_impl(impulse_response, samples)
    }

    fn remove_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        self.remove_impulse_response_impl(impulse_response)
    }

    fn load_hrtf_profile(&self, profile: SoundHrtfProfileDescriptor) -> Result<(), SoundError> {
        self.load_hrtf_profile_impl(profile)
    }

    fn remove_hrtf_profile(&self, profile_id: &str) -> Result<(), SoundError> {
        self.remove_hrtf_profile_impl(profile_id)
    }

    fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
        self.hrtf_profiles_impl()
    }

    fn set_ray_tracing_convolution_status(
        &self,
        status: SoundRayTracingConvolutionStatus,
    ) -> Result<(), SoundError> {
        self.set_ray_tracing_convolution_status_impl(status)
    }

    fn submit_ray_traced_impulse_response(
        &self,
        descriptor: SoundRayTracedImpulseResponseDescriptor,
    ) -> Result<(), SoundError> {
        self.submit_ray_traced_impulse_response_impl(descriptor)
    }

    fn ray_traced_impulse_responses(
        &self,
    ) -> Result<Vec<SoundRayTracedImpulseResponseDescriptor>, SoundError> {
        self.ray_traced_impulse_responses_impl()
    }

    fn clear_ray_traced_impulse_response(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        self.clear_ray_traced_impulse_response_impl(impulse_response)
    }
}
