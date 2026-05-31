use zircon_runtime::core::framework::sound::{SoundError, SoundMixerSnapshot};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn mixer_snapshot_impl(
        &self,
    ) -> Result<SoundMixerSnapshot, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .snapshot())
    }
}
