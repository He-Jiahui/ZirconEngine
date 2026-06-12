use super::super::super::super::*;

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
