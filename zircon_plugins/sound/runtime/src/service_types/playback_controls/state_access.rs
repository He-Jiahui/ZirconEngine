use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackId};

use crate::engine::ActivePlayback;

use super::super::DefaultSoundManager;

pub(super) fn with_active_playback_mut(
    manager: &DefaultSoundManager,
    playback: SoundPlaybackId,
    update: impl FnOnce(&mut ActivePlayback),
) -> Result<(), SoundError> {
    let mut state = manager.state.lock().expect("sound state mutex poisoned");
    let active = state
        .playbacks
        .get_mut(&playback)
        .ok_or(SoundError::UnknownPlayback { playback })?;
    update(active);
    Ok(())
}
