use super::super::super::*;

#[test]
fn external_7_1_block_downmixes_to_mono_without_lfe() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.mono"),
            backend: "software-test".to_string(),
            display_name: "External Mono Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            channel_layout: AudioChannelLayout::mono(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.7_1-mono-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 8,
                channel_layout: AudioChannelLayout::surround_7_1(),
                samples: vec![0.02, 0.04, 0.06, 9.0, 0.08, 0.10, 0.12, 0.14],
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
    let center = 0.06 * std::f32::consts::FRAC_1_SQRT_2;
    let left = 0.02 + center + (0.12 * std::f32::consts::FRAC_1_SQRT_2) + (0.08 * 0.5);
    let right = 0.04 + center + (0.14 * std::f32::consts::FRAC_1_SQRT_2) + (0.10 * 0.5);

    assert_eq!(mix.channel_count, 1);
    assert_eq!(mix.channel_layout, AudioChannelLayout::mono());
    assert_samples_near(&mix.samples, &[(left + right) * 0.5]);
}
