use super::super::super::*;

#[test]
fn sidechain_compressor_ducks_target_track_from_another_track() {
    let sound = DefaultSoundManager::default();
    let target_clip = sound.insert_clip_for_test(test_clip("res://sound/pad.wav", &[0.5, 0.5]));
    let key_clip = sound.insert_clip_for_test(test_clip("res://sound/kick.wav", &[0.5, 0.5]));
    let target = SoundTrackId::new(2);
    let key = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(target, "Pad"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Kick Sidechain"))
        .unwrap();
    sound
        .add_or_update_effect(
            target,
            SoundEffectDescriptor::new(
                SoundEffectId::new(2),
                "Sidechain Compressor",
                SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -18.0,
                    ratio: 8.0,
                    attack_ms: 1.0,
                    release_ms: 50.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(
                        zircon_runtime::core::framework::sound::SoundSidechainInput {
                            track: key,
                            pre_effects: true,
                        },
                    ),
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(
            target_clip,
            SoundPlaybackSettings {
                output_track: target,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(
            key_clip,
            SoundPlaybackSettings {
                output_track: key,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert!(mix.samples[0] > 0.5);
    assert!(mix.samples[0] < 1.0);
}
