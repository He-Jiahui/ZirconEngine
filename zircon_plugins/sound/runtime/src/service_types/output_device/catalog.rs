use zircon_runtime::core::framework::sound::{
    SoundBackendCapability, SoundError, SoundOutputDeviceInfo,
};

use super::super::DefaultSoundManager;
use crate::output::{available_output_backends, available_output_devices};

impl DefaultSoundManager {
    pub(in crate::service_types) fn available_output_devices_impl(
        &self,
    ) -> Result<Vec<SoundOutputDeviceInfo>, SoundError> {
        Ok(available_output_devices(&self.config()))
    }

    pub(in crate::service_types) fn available_output_backends_impl(
        &self,
    ) -> Result<Vec<SoundBackendCapability>, SoundError> {
        Ok(available_output_backends())
    }
}
