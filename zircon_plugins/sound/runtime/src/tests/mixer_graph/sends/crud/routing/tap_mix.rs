use super::super::super::super::super::*;

pub(super) fn render_track_send_tap_mix(pre_effects: bool) -> Vec<f32> {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/send-tap.wav", &[0.5]));
    let music = SoundTrackId::new(2);
    let aux = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(aux, "Aux"))
        .unwrap();
    sound
        .add_or_update_effect(
            music,
            SoundEffectDescriptor::new(
                SoundEffectId::new(4),
                "Music Gain",
                SoundEffectKind::Gain(SoundGainEffect { gain: 0.25 }),
            ),
        )
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 1.0,
                pre_effects,
            },
        )
        .unwrap();
    sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: music,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    sound.render_mix(1).unwrap().samples
}
