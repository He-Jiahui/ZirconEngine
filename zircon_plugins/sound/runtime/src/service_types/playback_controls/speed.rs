use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use super::super::{playback_validation::validate_playback_speed, DefaultSoundManager};
use super::state_access::with_active_playback_mut;

impl DefaultSoundManager {
    pub(in crate::service_types) fn set_playback_speed_impl(
        &self,
        playback: SoundPlaybackId,
        speed: f32,
    ) -> Result<(), SoundError> {
        let speed = validate_playback_speed(speed)?;
        with_active_playback_mut(self, playback, |active| {
            active.speed = speed;
        })
    }
}
