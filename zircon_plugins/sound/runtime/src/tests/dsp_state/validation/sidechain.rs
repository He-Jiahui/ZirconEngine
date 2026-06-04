use super::super::super::*;

#[test]
fn effect_update_revalidates_sidechain_track_references_and_cycles() {
    let sound = DefaultSoundManager::default();
    let key = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Key"))
        .unwrap();

    assert!(matches!(
        sound
            .add_or_update_effect(
                SoundTrackId::master(),
                test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -12.0,
                    ratio: 2.0,
                    attack_ms: 1.0,
                    release_ms: 10.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(SoundSidechainInput {
                        track: SoundTrackId::new(999),
                        pre_effects: true,
                    }),
                })),
            )
            .unwrap_err(),
        SoundError::UnknownTrack { .. }
    ));

    assert!(sound
        .add_or_update_effect(
            key,
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 2.0,
                attack_ms: 1.0,
                release_ms: 10.0,
                makeup_gain_db: 0.0,
                sidechain: Some(SoundSidechainInput {
                    track: key,
                    pre_effects: false,
                }),
            })),
        )
        .unwrap_err()
        .to_string()
        .contains("post-effect sidechain"));

    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(SoundEffectKind::Compressor(SoundCompressorEffect {
                threshold_db: -12.0,
                ratio: 2.0,
                attack_ms: 1.0,
                release_ms: 10.0,
                makeup_gain_db: 0.0,
                sidechain: Some(SoundSidechainInput {
                    track: key,
                    pre_effects: true,
                }),
            })),
        )
        .unwrap();
}
