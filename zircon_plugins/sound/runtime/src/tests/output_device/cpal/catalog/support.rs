use super::super::super::super::*;
use zircon_runtime::core::framework::sound::SoundBackendCapability;

pub(super) fn cpal_backend(sound: &DefaultSoundManager) -> SoundBackendCapability {
    sound
        .available_output_backends()
        .unwrap()
        .into_iter()
        .find(|backend| backend.backend == "cpal")
        .expect("cpal backend should be listed with cpal-backend feature")
}
