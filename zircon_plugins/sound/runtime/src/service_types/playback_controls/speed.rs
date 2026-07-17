use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::{playback_validation::validate_playback_speed, DefaultSoundManager};
use crate::poison_recovery::lock_recover;

impl DefaultSoundManager {
    pub(in crate::service_types) fn set_playback_speed_impl(
        &self,
        playback: SoundPlaybackId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        let mut state = lock_recover(&self.state);
        state.poll_kira_completions();
        if !state.playbacks.contains_key(&playback) {
            return Err(SoundError::UnknownPlayback { playback });
        }
        state.kira.set_playback_rate(playback, speed)?;
        state
            .playbacks
            .get_mut(&playback)
            .ok_or(SoundError::UnknownPlayback { playback })?
            .speed = speed;
        Ok(())
    }
}
