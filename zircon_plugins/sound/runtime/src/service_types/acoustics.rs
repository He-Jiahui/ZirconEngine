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
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        validate_listener_descriptor(&state, &listener)?;
        state.kira.ensure_provider_not_retiring()?;
        let id = listener.id;
        let previous = state.listeners.insert(id, listener);
        if let Err(error) = state.refresh_source_environments() {
            if let Some(previous) = previous {
                state.listeners.insert(id, previous);
            } else {
                state.listeners.remove(&id);
            }
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn remove_listener_impl(&self, listener: SoundListenerId) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        state.kira.ensure_provider_not_retiring()?;
        let previous = state
            .listeners
            .remove(&listener)
            .ok_or(SoundError::UnknownListener { listener })?;
        if let Err(error) = state.refresh_source_environments() {
            state.listeners.insert(listener, previous);
            return Err(error);
        }
        Ok(())
    }

    pub(super) fn update_volume_impl(
        &self,
        volume: SoundVolumeDescriptor,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        validate_volume_descriptor(&volume)?;
        state.volumes.insert(volume.id, volume);
        Ok(())
    }

    pub(super) fn remove_volume_impl(&self, volume: SoundVolumeId) -> Result<(), SoundError> {
        crate::poison_recovery::lock_recover(&self.state)
            .volumes
            .remove(&volume)
            .map(|_| ())
            .ok_or(SoundError::UnknownVolume { volume })
    }
}
