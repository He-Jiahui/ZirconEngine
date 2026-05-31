use zircon_runtime::core::framework::sound::{SoundError, SoundTrackDescriptor, SoundTrackId};

use super::super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(in crate::service_types) fn add_or_update_track_impl(
        &self,
        track: SoundTrackDescriptor,
    ) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .add_or_replace_track(track)
    }

    pub(in crate::service_types) fn remove_track_impl(
        &self,
        track: SoundTrackId,
    ) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .remove_track(track)
    }
}
