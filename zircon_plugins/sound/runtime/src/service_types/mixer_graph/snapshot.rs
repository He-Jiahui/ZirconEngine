use zircon_runtime::core::framework::sound::{SoundError, SoundMixerSnapshot};

use super::super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn mixer_snapshot_impl(
        &self,
    ) -> Result<SoundMixerSnapshot, SoundError> {
        Ok(lock_recover(&self.state).snapshot())
    }
}
