use super::super::super::super::*;
use super::support::{configure_started_test_output, play_output_test_clip};

#[test]
fn configured_output_device_updates_render_status() {
    let sound = DefaultSoundManager::default();
    let descriptor = configure_started_test_output(&sound);
    play_output_test_clip(&sound);

    sound.render_output_device_block().unwrap();

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, descriptor);
    assert_eq!(status.state, SoundOutputDeviceState::Started);
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.underrun_count, 0);
    assert_eq!(status.last_error, None);
}
