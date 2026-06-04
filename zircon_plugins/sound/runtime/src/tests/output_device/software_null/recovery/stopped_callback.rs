use super::super::super::super::*;

#[test]
fn software_null_backend_rejects_stopped_callback() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .pull_output_backend_callback()
        .unwrap_err()
        .to_string()
        .contains("stopped"));
}
