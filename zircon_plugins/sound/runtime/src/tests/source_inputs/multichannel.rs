use super::super::*;

#[test]
fn external_surround_block_downmixes_to_stereo_without_lfe() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.surround-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 6,
                channel_layout: SoundChannelLayout::surround_5_1(),
                samples: vec![0.10, 0.20, 0.30, 9.0, 0.40, 0.50],
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
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}

#[test]
fn external_quad_block_downmixes_rear_pair_into_stereo_front_pair() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.quad-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 4,
                channel_layout: SoundChannelLayout::quad(),
                samples: vec![0.10, 0.20, 0.40, 0.50],
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

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}

#[test]
fn external_discrete_4_block_folds_overflow_pair_into_stereo_output() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.discrete-4-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(
                48_000,
                SoundChannelLayout::discrete(4),
                vec![0.10, 0.20, 0.40, 0.50],
            ),
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}

#[test]
fn external_5_0_block_uses_named_speaker_downmix_from_channel_count() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.5_0-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 5,
                channel_layout: SoundChannelLayout::surround_5_0(),
                samples: vec![0.10, 0.20, 0.30, 0.40, 0.50],
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
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}

#[test]
fn external_7_1_block_folds_side_pair_into_5_1_rear_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.surround.5_1"),
            backend: "software-test".to_string(),
            display_name: "External 5.1 Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.7_1-bed");
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

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 1.0, 1.20]);
}

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
            channel_layout: SoundChannelLayout::surround_5_1(),
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
                channel_layout: SoundChannelLayout::surround_5_1_side(),
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

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 0.60, 0.70]);
}

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

#[test]
fn external_5_1_block_folds_center_into_quad_front_pair_without_lfe() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.quad"),
            backend: "software-test".to_string(),
            display_name: "External Quad Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 4,
            channel_layout: SoundChannelLayout::quad(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.5_1-center-to-quad-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(
                48_000,
                SoundChannelLayout::surround_5_1(),
                vec![0.10, 0.20, 0.30, 9.0, 0.40, 0.50],
            ),
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::quad());
    assert_samples_near(&mix.samples, &[0.10 + center, 0.20 + center, 0.40, 0.50]);
}

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
            channel_layout: SoundChannelLayout::mono(),
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
                channel_layout: SoundChannelLayout::surround_7_1(),
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
    assert_eq!(mix.channel_layout, SoundChannelLayout::mono());
    assert_samples_near(&mix.samples, &[(left + right) * 0.5]);
}
