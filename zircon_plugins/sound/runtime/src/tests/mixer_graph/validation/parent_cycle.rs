use super::super::super::*;

#[test]
fn mixer_graph_rejects_parent_track_cycles() {
    let sound = DefaultSoundManager::default();
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
}
