use super::super::super::super::*;
use super::support::{configure_started_software_null_output, play_software_null_test_clip};

#[test]
fn software_null_backend_callback_updates_output_status() {
    let sound = DefaultSoundManager::default();
    configure_started_software_null_output(&sound);
    play_software_null_test_clip(&sound);

    sound.pull_output_backend_callback().unwrap();

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.callback_count, 1);
    assert_eq!(status.last_callback_sequence, Some(0));
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.latency.requested_latency_blocks, 2);
    assert_eq!(status.latency.estimated_latency_frames, 4);
    assert!(status.diagnostics.is_empty());
}
