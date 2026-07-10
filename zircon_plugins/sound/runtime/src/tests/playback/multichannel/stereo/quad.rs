use super::super::super::super::*;

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

    assert_eq!(mix.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}
