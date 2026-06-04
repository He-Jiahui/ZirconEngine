use super::super::super::*;

#[test]
fn low_pass_filter_keeps_state_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/stateful-filter.wav", &[1.0]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Filter(SoundFilterEffect {
                mode: SoundFilterMode::LowPass,
                cutoff_hz: 1_000.0,
                resonance: 0.0,
                gain_db: 0.0,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let first = sound.render_mix(1).unwrap().samples;
    let second = sound.render_mix(1).unwrap().samples;

    assert!(first[0] > 0.0 && first[0] < 0.05);
    assert!(second[0] > first[0]);
    assert_sample_near(first[0], first[1]);
    assert_sample_near(second[0], second[1]);
}
