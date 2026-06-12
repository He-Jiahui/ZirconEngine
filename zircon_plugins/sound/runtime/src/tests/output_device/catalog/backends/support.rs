use super::super::super::super::*;
use zircon_runtime::core::framework::sound::SoundBackendCapability;

pub(super) fn software_null_backend(sound: &DefaultSoundManager) -> SoundBackendCapability {
    sound
        .available_output_backends()
        .unwrap()
        .into_iter()
        .find(|backend| backend.backend == "software-null")
        .expect("software-null backend should be listed")
}
