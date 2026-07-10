use super::super::super::super::*;

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
            channel_layout: AudioChannelLayout::surround_5_1(),
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
    assert_eq!(mix.channel_layout, AudioChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.25, 0.75, 0.0, 0.0, 0.0, 0.0]);
}
