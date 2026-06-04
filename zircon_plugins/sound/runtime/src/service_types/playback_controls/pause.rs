use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::DefaultSoundManager;
use super::state_access::with_active_playback_mut;

impl DefaultSoundManager {
    pub(in crate::service_types) fn pause_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.paused = true;
        })
    }

    pub(in crate::service_types) fn resume_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.paused = false;
        })
    }

    pub(in crate::service_types) fn toggle_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.paused = !active.paused;
        })
    }
}
