use super::super::super::super::*;

#[test]
fn track_send_removal_reports_unknown_send_and_target_track() {
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
                gain: 0.5,
                pre_effects: false,
            },
        )
        .unwrap();

    sound.remove_track_send(music, aux).unwrap();
    assert!(sound
        .remove_track_send(music, aux)
        .unwrap_err()
        .to_string()
        .contains("unknown send"));
    assert!(sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: SoundTrackId::new(99),
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("unknown track"));
}
