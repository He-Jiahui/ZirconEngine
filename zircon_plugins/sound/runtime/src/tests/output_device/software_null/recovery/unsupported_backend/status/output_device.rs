use super::super::super::super::super::super::*;
use super::super::support::{assert_not_available_error, unsupported_native_descriptor};

#[test]
fn unsupported_backend_reports_output_device_diagnostics() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(unsupported_native_descriptor())
        .unwrap_err();
    assert_not_available_error(error);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.state, SoundOutputDeviceState::Stopped);
    assert!(status
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("not available")));
}
