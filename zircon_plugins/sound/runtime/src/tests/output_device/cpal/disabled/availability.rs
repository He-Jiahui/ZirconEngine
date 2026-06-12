use super::super::super::super::*;

#[test]
fn cpal_backend_is_absent_when_not_compiled() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .available_output_backends()
        .unwrap()
        .iter()
        .all(|backend| backend.backend != "cpal"));
    assert!(sound
        .available_output_devices()
        .unwrap()
        .iter()
        .all(|device| device.descriptor.backend != "cpal"));
}
