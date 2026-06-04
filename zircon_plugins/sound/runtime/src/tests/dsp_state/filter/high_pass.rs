use super::super::super::*;

#[test]
fn high_pass_filter_rejects_dc() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/high-pass-dc.wav", &vec![0.5; 128]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::HighPass,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(128).unwrap().samples;
    let first_left = mix[0].abs();
    let last_left = mix[mix.len() - 2].abs();

    assert!(first_left > 0.25);
    assert!(last_left < first_left * 0.1);
}
