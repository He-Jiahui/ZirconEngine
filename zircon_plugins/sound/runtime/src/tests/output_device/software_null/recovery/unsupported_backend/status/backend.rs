use super::super::super::super::super::super::*;
use super::super::support::{assert_not_available_error, unsupported_native_descriptor};

#[test]
fn unsupported_backend_updates_backend_status() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(unsupported_native_descriptor())
        .unwrap_err();
    assert_not_available_error(error);

    let status = sound.backend_status();
    assert_eq!(status.requested_backend, "native-missing");
    assert_eq!(status.active_backend, None);
    assert_eq!(status.state, SoundBackendState::Unavailable);
    assert!(status
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("not available"));
    assert_eq!(sound.backend_name(), "native-missing");
}
