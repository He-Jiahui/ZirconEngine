use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn mute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        if !state.playbacks.contains_key(&playback) {
            return Err(SoundError::UnknownPlayback { playback });
        }
        state.kira.set_volume(playback, 0.0)?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .muted = true;
        Ok(())
    }

    pub(in crate::service_types) fn unmute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let gain = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .gain;
        state.kira.set_volume(playback, gain)?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .muted = false;
        Ok(())
    }

    pub(in crate::service_types) fn toggle_mute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let active = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let (muted, gain) = (active.muted, active.gain);
        state
            .kira
            .set_volume(playback, if muted { gain } else { 0.0 })?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .muted = !muted;
        Ok(())
    }
}
