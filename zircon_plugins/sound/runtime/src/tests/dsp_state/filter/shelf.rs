use super::super::super::*;

#[test]
fn shelf_filter_uses_gain_db() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/low-shelf-gain.wav",
        &vec![0.25; 512],
    ));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::LowShelf,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 6.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(512).unwrap().samples;
    let settled_left = mix[mix.len() - 2];

    assert!(settled_left > 0.35);
    assert_sample_near(settled_left, mix[mix.len() - 1]);
}
