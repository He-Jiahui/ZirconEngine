use zircon_runtime::core::framework::sound::{
    SoundError, SoundListenerDescriptor, SoundListenerId, SoundVolumeDescriptor, SoundVolumeId,
};

use crate::descriptor_validation::listener::validate_listener_descriptor;
use crate::descriptor_validation::volume::validate_volume_descriptor;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn update_listener_impl(
        &self,
        listener: SoundListenerDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_listener_descriptor(&state, &listener)?;
        state.listeners.insert(listener.id, listener);
        Ok(())
    }

    pub(super) fn remove_listener_impl(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .listeners
            .remove(&listener)
            .map(|_| ())
            .ok_or(SoundError::UnknownListener { listener })
    }

    pub(super) fn update_volume_impl(
        &self,
        volume: SoundVolumeDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        validate_volume_descriptor(&volume)?;
        state.volumes.insert(volume.id, volume);
        Ok(())
    }

    pub(super) fn remove_volume_impl(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .volumes
            .remove(&volume)
            .map(|_| ())
            .ok_or(SoundError::UnknownVolume { volume })
    }
}
