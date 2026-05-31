use zircon_runtime::core::framework::sound::{SoundError, SoundImpulseResponseId};

use crate::ray_tracing::status::refresh_ray_tracing_status;

use super::DefaultSoundManager;

impl DefaultSoundManager {
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
}
