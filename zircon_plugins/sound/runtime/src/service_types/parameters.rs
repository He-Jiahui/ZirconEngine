use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn set_parameter_impl(
        &self,
        parameter: SoundParameterId,
        value: f32,
    ) -> Result<(), SoundError> {
        if !value.is_finite() {
            return Err(SoundError::InvalidParameter(format!(
                "parameter {} must be finite",
                parameter.as_str()
            )));
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .insert(parameter, value);
        Ok(())
    }

    pub(super) fn parameter_value_impl(
        &self,
        parameter: &SoundParameterId,
    ) -> Result<f32, SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .get(parameter)
            .copied()
            .ok_or_else(|| SoundError::UnknownParameter {
                parameter: parameter.clone(),
            })
    }
}
