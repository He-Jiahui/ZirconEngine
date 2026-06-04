use super::super::*;

#[test]
fn track_solo_mutes_non_solo_direct_inputs_but_keeps_route_to_master() {
    let sound = DefaultSoundManager::default();
    let solo_clip = sound.insert_clip_for_test(test_clip("res://sound/solo.wav", &[0.5]));
    let muted_clip = sound.insert_clip_for_test(test_clip("res://sound/non-solo.wav", &[0.5]));
    let master_clip = sound.insert_clip_for_test(test_clip("res://sound/master.wav", &[0.25]));
    let solo = SoundTrackId::new(2);
    let muted = SoundTrackId::new(3);
    let mut solo_track = SoundTrackDescriptor::child(solo, "Solo");
    solo_track.controls.solo = true;
    sound.add_or_update_track(solo_track).unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(muted, "Muted"))
        .unwrap();
    sound
        .play_clip(
            solo_clip,
            SoundPlaybackSettings {
                output_track: solo,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(
            muted_clip,
            SoundPlaybackSettings {
                output_track: muted,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(master_clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5]);
}
