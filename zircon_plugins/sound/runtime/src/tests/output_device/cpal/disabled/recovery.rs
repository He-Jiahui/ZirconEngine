use super::super::super::super::*;
use super::support::{cpal_disabled_descriptor, software_null_recovery_descriptor};

#[test]
fn cpal_disabled_backend_recovers_to_software_null() {
    let sound = DefaultSoundManager::default();

    sound
        .configure_output_device(cpal_disabled_descriptor())
        .unwrap_err();
    sound
        .configure_output_device(software_null_recovery_descriptor())
        .unwrap();

    assert_eq!(sound.backend_status().state, SoundBackendState::Ready);
}
