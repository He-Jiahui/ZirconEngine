use super::super::super::*;

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
