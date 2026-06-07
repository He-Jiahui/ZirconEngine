use super::super::*;

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
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 0,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("block size"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.bad-layout"),
            backend: "software-test".to_string(),
            display_name: "Bad Layout Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("channel layout"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.bad-speakers"),
            backend: "software-test".to_string(),
            display_name: "Bad Speaker Metadata Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout {
                name: "stereo".to_string(),
                channel_count: 2,
                speakers: vec![
                    SoundSpeakerChannel::FrontRight,
                    SoundSpeakerChannel::FrontLeft,
                ],
            },
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("canonical speaker metadata"));
}
