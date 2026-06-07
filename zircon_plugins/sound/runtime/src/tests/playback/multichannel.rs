use super::super::*;

#[test]
fn stereo_clip_feeds_front_pair_without_filling_surround_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.playback.surround"),
            backend: "software-test".to_string(),
            display_name: "Playback Surround Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_stereo_clip_with_rate(
        "res://sound/stereo-front-bed.wav",
        48_000,
        &[0.25, 0.75],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_count, 6);
    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.25, 0.75, 0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn quad_clip_downmixes_rear_pair_into_stereo_front_pair() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/quad-bed.wav",
        48_000,
        4,
        &[0.10, 0.20, 0.40, 0.50],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}

#[test]
fn discrete_4_clip_folds_overflow_pair_into_stereo_output() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_layout(
        "res://sound/discrete-4-bed.wav",
        48_000,
        SoundChannelLayout::discrete(4),
        &[0.10, 0.20, 0.40, 0.50],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}

#[test]
fn surround_5_0_clip_uses_named_speaker_downmix_from_channel_count() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/surround-5-0-bed.wav",
        48_000,
        5,
        &[0.10, 0.20, 0.30, 0.40, 0.50],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}

#[test]
fn surround_5_1_side_clip_uses_asset_declared_layout_for_stereo_downmix() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_layout(
        "res://sound/surround-5-1-side-bed.wav",
        48_000,
        SoundChannelLayout::surround_5_1_side(),
        &[0.10, 0.20, 0.30, 9.0, 0.60, 0.70],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(
        &mix.samples,
        &[
            0.10 + center + (0.60 * std::f32::consts::FRAC_1_SQRT_2),
            0.20 + center + (0.70 * std::f32::consts::FRAC_1_SQRT_2),
        ],
    );
}

#[test]
fn surround_7_1_clip_folds_side_pair_into_5_1_rear_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.playback.surround.5_1"),
            backend: "software-test".to_string(),
            display_name: "Playback 5.1 Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/side-rear-bed.wav",
        48_000,
        8,
        &[0.10, 0.20, 0.30, 9.0, 0.40, 0.50, 0.60, 0.70],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 1.0, 1.20]);
}

#[test]
fn surround_7_1_clip_folds_rear_pair_into_5_1_side_bed() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.playback.surround.5_1_side"),
            backend: "software-test".to_string(),
            display_name: "Playback 5.1 Side Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1_side(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/rear-side-bed.wav",
        48_000,
        8,
        &[0.10, 0.20, 0.30, 9.0, 0.40, 0.50, 0.60, 0.70],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1_side());
    assert_samples_near(&mix.samples, &[0.10, 0.20, 0.30, 9.0, 1.0, 1.20]);
}

#[test]
fn surround_5_1_clip_folds_center_into_quad_front_pair_without_lfe() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.playback.quad"),
            backend: "software-test".to_string(),
            display_name: "Playback Quad Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 4,
            channel_layout: SoundChannelLayout::quad(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/center-to-quad-bed.wav",
        48_000,
        6,
        &[0.10, 0.20, 0.30, 9.0, 0.40, 0.50],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::quad());
    assert_samples_near(&mix.samples, &[0.10 + center, 0.20 + center, 0.40, 0.50]);
}

#[test]
fn surround_7_1_clip_downmixes_to_mono_without_lfe() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.playback.mono"),
            backend: "software-test".to_string(),
            display_name: "Playback Mono Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            channel_layout: SoundChannelLayout::mono(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/mono-fold-down-bed.wav",
        48_000,
        8,
        &[0.02, 0.04, 0.06, 9.0, 0.08, 0.10, 0.12, 0.14],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();
    let center = 0.06 * std::f32::consts::FRAC_1_SQRT_2;
    let left = 0.02 + center + (0.12 * std::f32::consts::FRAC_1_SQRT_2) + (0.08 * 0.5);
    let right = 0.04 + center + (0.14 * std::f32::consts::FRAC_1_SQRT_2) + (0.10 * 0.5);

    assert_eq!(mix.channel_count, 1);
    assert_eq!(mix.channel_layout, SoundChannelLayout::mono());
    assert_samples_near(&mix.samples, &[(left + right) * 0.5]);
}
