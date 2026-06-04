use super::super::super::*;

#[test]
fn mixer_graph_routes_custom_track_through_effect_chain_to_master() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/tone.wav", &[1.0, 1.0]));
    let music = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_effect(
            music,
            SoundEffectDescriptor::new(
                SoundEffectId::new(1),
                "Music Gain",
                SoundEffectKind::Gain(SoundGainEffect { gain: 0.5 }),
            ),
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

    let mix = sound.render_mix(2).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5, 0.5, 0.5]);
    assert!(sound
        .mixer_snapshot()
        .unwrap()
        .meters
        .iter()
        .any(|meter| meter.track == music && meter.peak_left == 0.5));
}
