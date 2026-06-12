use super::super::super::super::*;
use super::support::cpal_disabled_descriptor;

#[test]
fn cpal_backend_rejects_configuration_when_not_compiled() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(cpal_disabled_descriptor())
        .unwrap_err();
    assert!(error.to_string().contains("cpal-backend"));
    assert_eq!(sound.backend_status().requested_backend, "cpal");
    assert_eq!(sound.backend_status().state, SoundBackendState::Unavailable);
}
