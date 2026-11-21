use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::DefaultSoundManager;
use crate::engine::ClipCursorOwner;
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn pause_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        self.update_playback_pause(playback, Some(true))
    }

    pub(in crate::service_types) fn resume_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        self.update_playback_pause(playback, Some(false))
    }

    pub(in crate::service_types) fn toggle_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        self.update_playback_pause(playback, None)
    }

    fn update_playback_pause(
        &self,
        playback: SoundPlaybackId,
        requested: Option<bool>,
    ) -> Result<(), SoundError> {
        let mut state = lock_recover(&self.state);
        state.kira.ensure_control_available()?;
        state.poll_kira_completions();
        let state = &mut *state;
        let active = state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?;
        let paused = requested.unwrap_or(!active.paused);
        if paused == active.paused {
            use kira::sound::PlaybackState::{Paused, Pausing, Playing, Resuming};
            // Don't restart the fade on an already paused voice. If feedback disagrees,
            // still submit the control: the logical flag is not a mixer acknowledgement.
            let actual = state.kira.playback_state(playback)?;
            if matches!(
                (paused, actual),
                (true, Paused | Pausing) | (false, Playing | Resuming)
            ) {
                return Ok(());
            }
        }
        if paused {
            state.kira.pause(playback)?;
        } else if active.cursor_owner == ClipCursorOwner::PausedSeek {
            let request = state.kira.resume_after_paused_seek(playback)?;
            active.cursor_owner = ClipCursorOwner::AwaitingMixer(request);
        } else {
            state.kira.resume(playback)?;
        }
        active.paused = paused;
        Ok(())
    }
}
