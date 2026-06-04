use super::super::super::*;

#[test]
fn removing_track_reroutes_active_playbacks_before_finished_events() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/remove-track.wav", &[0.5]));
    let music = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    let playback = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: music,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    assert_eq!(sound.playback_status(playback).unwrap().output_track, music);
    sound.remove_track(music).unwrap();
    assert_eq!(
        sound.playback_status(playback).unwrap().output_track,
        SoundTrackId::master()
    );
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.5, 0.5]);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    let finished = sound.drain_finished_playbacks().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].playback, playback);
    assert_eq!(finished[0].output_track, SoundTrackId::master());
}
