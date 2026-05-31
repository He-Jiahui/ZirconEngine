use zircon_runtime::core::framework::sound::SoundError;

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn start_output_device_impl(&self) -> Result<(), SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .start_with_engine(self.state.clone(), self.config.clone())?;
        Ok(())
    }

    pub(in crate::service_types) fn stop_output_device_impl(&self) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .output_device
            .stop();
        Ok(())
    }
}
