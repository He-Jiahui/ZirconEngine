use super::super::super::super::*;

#[test]
fn external_5_1_side_block_uses_explicit_layout_when_output_has_rear_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.side-to-rear"),
            backend: "software-test".to_string(),
            display_name: "External Side To Rear Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: AudioChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.5_1-side-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 6,
                channel_layout: AudioChannelLayout::surround_5_1_side(),
                samples: vec![0.10, 0.20, 0.30, 9.0, 0.60, 0.70],
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

    assert_eq!(mix.channel_layout, AudioChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 0.60, 0.70]);
}
