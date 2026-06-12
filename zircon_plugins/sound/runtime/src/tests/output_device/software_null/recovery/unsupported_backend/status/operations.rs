use super::super::super::super::super::super::*;
use super::super::support::{assert_not_available_error, unsupported_native_descriptor};

#[test]
fn unsupported_backend_rejects_start_and_callback_operations() {
    let sound = DefaultSoundManager::default();

    let error = sound
        .configure_output_device(unsupported_native_descriptor())
        .unwrap_err();
    assert_not_available_error(error);

    assert!(sound
        .start_output_device()
        .unwrap_err()
        .to_string()
        .contains("not available"));
    assert!(sound
        .pull_output_backend_callback()
        .unwrap_err()
        .to_string()
        .contains("not available"));
}
