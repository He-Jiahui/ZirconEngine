use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::DefaultSoundManager;
use super::state_access::with_active_playback_mut;

impl DefaultSoundManager {
    pub(in crate::service_types) fn mute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.muted = true;
        })
    }

    pub(in crate::service_types) fn unmute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.muted = false;
        })
    }

    pub(in crate::service_types) fn toggle_mute_playback_impl(
        &self,
        playback: SoundPlaybackId,
    ) -> Result<(), SoundError> {
        with_active_playback_mut(self, playback, |active| {
            active.muted = !active.muted;
        })
    }
}
