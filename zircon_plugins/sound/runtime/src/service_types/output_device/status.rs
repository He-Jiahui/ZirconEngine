use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceStatus};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn output_device_status_impl(
        &self,
    ) -> Result<SoundOutputDeviceStatus, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .status())
    }
}
