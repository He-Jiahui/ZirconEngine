use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceStatus};

use super::super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn output_device_status_impl(
        &self,
    ) -> Result<SoundOutputDeviceStatus, SoundError> {
        Ok(lock_recover(&self.state).output_device.status())
    }

    #[cfg(test)]
    pub(crate) fn mark_output_device_started_for_test(&self) {
        lock_recover(&self.state).output_device.mark_started();
    }
}
