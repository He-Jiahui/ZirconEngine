use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::DefaultSoundManager;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn pause_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        if !state.playbacks.contains_key(&playback) {
            return Err(SoundError::UnknownPlayback { playback });
        }
        state.kira.pause(playback)?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .paused = true;
        Ok(())
    }

    pub(in crate::service_types) fn resume_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        if !state.playbacks.contains_key(&playback) {
            return Err(SoundError::UnknownPlayback { playback });
        }
        state.kira.resume(playback)?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .paused = false;
        Ok(())
    }

    pub(in crate::service_types) fn toggle_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        let paused = state
            .playbacks
            .get(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .paused;
        if paused {
            state.kira.resume(playback)?;
        } else {
            state.kira.pause(playback)?;
        }
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .paused = !paused;
        Ok(())
    }
}
