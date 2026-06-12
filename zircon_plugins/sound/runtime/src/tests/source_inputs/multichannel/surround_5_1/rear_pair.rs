use super::super::super::super::*;

#[test]
fn external_7_1_block_folds_rear_pair_into_5_1_side_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.surround.5_1_side"),
            backend: "software-test".to_string(),
            display_name: "External 5.1 Side Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1_side(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.7_1-side-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 8,
                channel_layout: SoundChannelLayout::surround_7_1(),
                samples: vec![0.10, 0.20, 0.30, 9.0, 0.40, 0.50, 0.60, 0.70],
            },
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1_side());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 1.0, 1.20]);
}
