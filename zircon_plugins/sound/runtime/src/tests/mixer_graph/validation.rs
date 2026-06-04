use super::super::*;

#[test]
fn mixer_graph_rejects_parent_cycles_and_missing_tracks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/cycle.wav", &[1.0]));
    let a = SoundTrackId::new(2);
    let b = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(a, "A"))
        .unwrap();
    let mut b_track = SoundTrackDescriptor::child(b, "B");
    b_track.parent = Some(a);
    sound.add_or_update_track(b_track).unwrap();
    let mut a_cycle = SoundTrackDescriptor::child(a, "A");
    a_cycle.parent = Some(b);

    let error = sound.add_or_update_track(a_cycle).unwrap_err();
    assert!(error.to_string().contains("cycle"));

    let missing = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: SoundTrackId::new(99),
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap_err();
    assert!(missing.to_string().contains("unknown track"));
}
