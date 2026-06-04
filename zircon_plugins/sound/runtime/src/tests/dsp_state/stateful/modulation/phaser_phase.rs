use super::super::super::super::*;

#[test]
fn phaser_lfo_phase_continues_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip =
        sound.insert_clip_for_test(test_clip("res://sound/stateful-phaser.wav", &[1.0, 1.0]));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Phaser(SoundPhaserEffect {
                rate_hz: 12_000.0,
                depth: 1.0,
                feedback: 0.0,
                phase_offset: 0.25,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.5, 0.5]);
}
