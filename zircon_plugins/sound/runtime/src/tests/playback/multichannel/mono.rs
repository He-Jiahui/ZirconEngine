use super::super::super::*;

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
            channel_layout: AudioChannelLayout::mono(),
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
    assert_eq!(mix.channel_layout, AudioChannelLayout::mono());
    assert_samples_near(&mix.samples, &[(left + right) * 0.5]);
}
