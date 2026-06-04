use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use crate::automation::values::ensure_finite_value;

use super::super::DefaultSoundManager;
use super::state_access::with_active_playback_mut;

impl DefaultSoundManager {
    pub(in crate::service_types) fn set_playback_gain_impl(
        &self,
        playback: SoundPlaybackId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback gain", gain)?;
        with_active_playback_mut(self, playback, |active| {
            active.gain = gain;
        })
    }
}
