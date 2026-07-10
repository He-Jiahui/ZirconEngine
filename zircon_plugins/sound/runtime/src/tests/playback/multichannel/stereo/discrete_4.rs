use super::super::super::super::*;

#[test]
fn discrete_4_clip_folds_overflow_pair_into_stereo_output() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_layout(
        "res://sound/discrete-4-bed.wav",
        48_000,
        AudioChannelLayout::discrete(4),
        &[0.10, 0.20, 0.40, 0.50],
    ));

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}
