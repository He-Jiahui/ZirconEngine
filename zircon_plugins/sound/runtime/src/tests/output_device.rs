use super::*;

#[test]
fn output_device_can_be_configured_started_and_pulled() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.test"),
        backend: "software-test".to_string(),
        display_name: "Software Test Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        block_size_frames: 2,
        latency_blocks: 2,
    };
    sound.configure_output_device(descriptor.clone()).unwrap();
    sound.start_output_device().unwrap();

    let clip = sound.insert_clip_for_test(test_clip("res://sound/output.wav", &[0.25, 0.5]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let block = sound.render_output_device_block().unwrap();
    assert_samples_near(&block.samples, &[0.25, 0.25, 0.5, 0.5]);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, descriptor);
    assert_eq!(status.state, SoundOutputDeviceState::Started);
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.underrun_count, 0);
    assert_eq!(status.last_error, None);
}

#[test]
fn output_device_updates_runtime_format_and_stops_on_reconfigure() {
    let sound = DefaultSoundManager::default();
    sound.start_output_device().unwrap();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.preview"),
            backend: "software-preview".to_string(),
            display_name: "Preview Output".to_string(),
            sample_rate_hz: 24_000,
            channel_count: 1,
            block_size_frames: 3,
            latency_blocks: 1,
        })
        .unwrap();

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.state, SoundOutputDeviceState::Stopped);
    assert_eq!(status.rendered_frames, 0);
    assert_eq!(sound.backend_status().sample_rate_hz, 24_000);
    assert_eq!(sound.backend_status().channel_count, 1);
    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.sample_rate_hz, 24_000);
    assert_eq!(snapshot.graph.channel_count, 1);
}

#[test]
fn output_device_rejects_invalid_descriptor_and_stopped_pull() {
    let sound = DefaultSoundManager::default();
    assert!(sound
        .render_output_device_block()
        .unwrap_err()
        .to_string()
        .contains("output device is stopped"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.bad"),
            backend: "software-test".to_string(),
            display_name: "Bad Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 0,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("block size"));
}
