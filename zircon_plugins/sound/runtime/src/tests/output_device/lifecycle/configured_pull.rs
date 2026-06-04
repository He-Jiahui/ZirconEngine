use super::super::super::*;

#[test]
fn output_device_can_be_configured_started_and_pulled() {
    let sound = DefaultSoundManager::default();
    let descriptor = SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.test"),
        backend: "software-test".to_string(),
        display_name: "Software Test Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: SoundChannelLayout::stereo(),
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
    assert_eq!(block.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&block.samples, &[0.25, 0.25, 0.5, 0.5]);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.descriptor, descriptor);
    assert_eq!(status.state, SoundOutputDeviceState::Started);
    assert_eq!(status.rendered_blocks, 1);
    assert_eq!(status.rendered_frames, 2);
    assert_eq!(status.underrun_count, 0);
    assert_eq!(status.last_error, None);
}
