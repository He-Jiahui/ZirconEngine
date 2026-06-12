use super::super::super::super::super::*;
use super::support::{software_null_retry_descriptor, unsupported_native_descriptor};

#[test]
fn software_null_backend_recovers_after_unsupported_backend() {
    let sound = DefaultSoundManager::default();

    sound
        .configure_output_device(unsupported_native_descriptor())
        .unwrap_err();
    sound
        .configure_output_device(software_null_retry_descriptor())
        .unwrap();

    let status = sound.backend_status();
    assert_eq!(status.requested_backend, "software-null");
    assert_eq!(status.active_backend.as_deref(), Some("software-null"));
    assert_eq!(status.state, SoundBackendState::Ready);
    assert_eq!(status.detail, None);
}
