use super::super::super::*;

#[test]
fn output_device_rejects_stopped_pull() {
    let sound = DefaultSoundManager::default();

    assert!(sound
        .render_output_device_block()
        .unwrap_err()
        .to_string()
        .contains("output device is stopped"));
}
