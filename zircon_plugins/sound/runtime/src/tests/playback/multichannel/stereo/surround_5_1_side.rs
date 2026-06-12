use super::super::super::super::*;

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
