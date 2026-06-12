use super::super::super::*;

pub(super) fn render_sidechain_tap_mix(pre_effects: bool) -> Vec<f32> {
    let sound = DefaultSoundManager::default();
    let target_clip =
        sound.insert_clip_for_test(test_clip("res://sound/sidechain-target.wav", &[0.5]));
    let key_clip = sound.insert_clip_for_test(test_clip("res://sound/sidechain-key.wav", &[0.5]));
    let target = SoundTrackId::new(2);
    let key = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(target, "Target"))
        .unwrap();
    let mut key_track = SoundTrackDescriptor::child(key, "Muted Key");
    key_track.controls.mute = true;
    sound.add_or_update_track(key_track).unwrap();
    sound
        .add_or_update_effect(
            target,
            SoundEffectDescriptor::new(
                SoundEffectId::new(77),
                "Sidechain Compressor",
                SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -18.0,
                    ratio: 8.0,
                    attack_ms: 1.0,
                    release_ms: 50.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(SoundSidechainInput {
                        track: key,
                        pre_effects,
                    }),
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
    sound.render_mix(1).unwrap().samples
}
