use zircon_runtime::core::framework::sound::{
    SoundError, SoundHrtfProfileDescriptor, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundListenerId, SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundVolumeDescriptor, SoundVolumeId,
};

use crate::descriptor_validation::{
    validate_hrtf_profile_descriptor, validate_listener_descriptor, validate_volume_descriptor,
};
use crate::ray_tracing::{
    clear_ray_traced_impulse_response, refresh_ray_tracing_status,
    submit_ray_traced_impulse_response, validate_ray_tracing_status,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn update_listener_impl(
        &self,
        listener: SoundListenerDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_listener_descriptor(&state, &listener)?;
        state.listeners.insert(listener.id, listener);
        Ok(())
    }

    pub(super) fn remove_listener_impl(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .listeners
            .remove(&listener)
            .map(|_| ())
            .ok_or(SoundError::UnknownListener { listener })
    }

    pub(super) fn update_volume_impl(
        &self,
        volume: SoundVolumeDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_volume_descriptor(&volume)?;
        state.volumes.insert(volume.id, volume);
        Ok(())
    }

    pub(super) fn remove_volume_impl(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .volumes
            .remove(&volume)
            .map(|_| ())
            .ok_or(SoundError::UnknownVolume { volume })
    }

    pub(super) fn set_impulse_response_impl(
        &self,
        impulse_response: SoundImpulseResponseId,
        samples: Vec<f32>,
    ) -> Result<(), SoundError> {
        if samples.is_empty() || samples.iter().any(|sample| !sample.is_finite()) {
            return Err(SoundError::InvalidParameter(
                "impulse response samples must be non-empty and finite".to_string(),
            ));
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .impulse_responses
            .insert(impulse_response, samples);
        Ok(())
    }

    pub(super) fn remove_impulse_response_impl(
        &self,
        impulse_response: SoundImpulseResponseId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .impulse_responses
            .remove(&impulse_response)
            .map(|_| ())
            .ok_or(SoundError::UnknownImpulseResponse { impulse_response })?;
        state.ray_traced_impulse_responses.remove(&impulse_response);
        let descriptors = state.ray_traced_impulse_responses.clone();
        refresh_ray_tracing_status(&mut state.ray_tracing, &descriptors);
        Ok(())
    }

    pub(super) fn load_hrtf_profile_impl(
        &self,
        profile: SoundHrtfProfileDescriptor,
    ) -> Result<(), SoundError> {
        validate_hrtf_profile_descriptor(&profile)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .insert(profile.profile_id.clone(), profile);
        state.hrtf_states.clear();
        Ok(())
    }

    pub(super) fn remove_hrtf_profile_impl(&self, profile_id: &str) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        state
            .hrtf_profiles
            .remove(profile_id)
            .map(|_| ())
            .ok_or_else(|| SoundError::UnknownHrtfProfile {
                profile_id: profile_id.to_string(),
            })?;
        state.hrtf_states.clear();
        Ok(())
    }

    pub(super) fn hrtf_profiles_impl(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError> {
        let mut profiles = self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .hrtf_profiles
            .values()
            .cloned()
            .collect::<Vec<_>>();
        profiles.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        Ok(profiles)
    }

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
