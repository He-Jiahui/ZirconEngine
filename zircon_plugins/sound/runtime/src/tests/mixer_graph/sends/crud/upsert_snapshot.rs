use super::super::super::super::*;

#[test]
fn track_send_upsert_replaces_existing_snapshot_gain() {
    let sound = DefaultSoundManager::default();
    let music = SoundTrackId::new(2);
    let aux = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(aux, "Aux"))
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 0.25,
                pre_effects: false,
            },
        )
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 0.5,
                pre_effects: false,
            },
        )
        .unwrap();

    let snapshot = sound.mixer_snapshot().unwrap();
    let music_track = snapshot
        .graph
        .tracks
        .iter()
        .find(|track| track.id == music)
        .unwrap();
    assert_eq!(music_track.sends.len(), 1);
    assert_sample_near(music_track.sends[0].gain, 0.5);
}
