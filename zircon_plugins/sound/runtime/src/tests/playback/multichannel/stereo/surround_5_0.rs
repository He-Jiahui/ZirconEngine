use super::super::super::super::*;

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

    assert_eq!(mix.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}
