use super::super::super::super::*;
use super::support::{configure_started_software_null_output, play_software_null_test_clip};

#[test]
fn software_null_backend_callback_reports_rendered_block() {
    let sound = DefaultSoundManager::default();
    configure_started_software_null_output(&sound);
    play_software_null_test_clip(&sound);

    let callback = sound.pull_output_backend_callback().unwrap();
    assert_eq!(callback.report.backend, "software-null");
    assert_eq!(callback.report.sequence_index, 0);
    assert_eq!(callback.report.requested_frames, 2);
    assert_eq!(callback.report.rendered_frames, 2);
    assert_eq!(callback.report.sample_count, 4);
    assert!(!callback.report.underrun);
    assert_eq!(callback.report.error, None);
    assert_samples_near(&callback.block.samples, &[0.25, 0.25, 0.5, 0.5]);
}
