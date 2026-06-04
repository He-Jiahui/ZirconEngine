use super::super::super::*;

#[test]
fn compressor_release_envelope_continues_across_render_blocks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/stateful-compressor.wav",
        &[1.0, 0.1],
    ));
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 20.0,
                attack_ms: 0.0,
                release_ms: 1000.0,
                makeup_gain_db: 0.0,
                sidechain: None,
            })),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let first = sound.render_mix(1).unwrap().samples;
    let second = sound.render_mix(1).unwrap().samples;

    assert!(first[0] < 0.5);
    assert!(second[0] < 0.05);
    assert_sample_near(second[0], second[1]);
}
