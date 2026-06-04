use super::super::super::*;

#[test]
fn software_null_backend_callback_reports_rendered_block() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.null"),
            backend: "software-null".to_string(),
            display_name: "Software Null Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 2,
            latency_blocks: 2,
        })
        .unwrap();
    sound.start_output_device().unwrap();

    let clip = sound.insert_clip_for_test(test_clip("res://sound/null-output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let callback = sound.pull_output_backend_callback().unwrap();
    assert_eq!(callback.report.backend, "software-null");
    assert_eq!(callback.report.sequence_index, 0);
    assert_eq!(callback.report.requested_frames, 2);
    assert_eq!(callback.report.rendered_frames, 2);
    assert_eq!(callback.report.sample_count, 4);
    assert!(!callback.report.underrun);
    assert_eq!(callback.report.error, None);
    assert_samples_near(&callback.block.samples, &[0.25, 0.25, 0.5, 0.5]);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.callback_count, 1);
    assert_eq!(status.last_callback_sequence, Some(0));
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.latency.requested_latency_blocks, 2);
    assert_eq!(status.latency.estimated_latency_frames, 4);
    assert!(status.diagnostics.is_empty());
}
