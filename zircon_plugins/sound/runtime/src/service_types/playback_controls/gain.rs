use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use crate::automation::values::ensure_finite_value;

use super::super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn set_playback_gain_impl(
        &self,
        playback: SoundPlaybackId,
        gain: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("playback gain", gain)?;
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let muted = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .muted;
        state
            .kira
            .set_volume(playback, if muted { 0.0 } else { gain })?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .gain = gain;
        Ok(())
    }
}
