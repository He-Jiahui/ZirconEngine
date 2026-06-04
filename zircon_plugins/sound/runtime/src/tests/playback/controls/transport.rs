use super::super::super::*;

#[test]
fn playback_pause_resume_and_toggle_update_transport_state() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/playback-lifecycle.wav",
        &[0.25, 0.5, 0.75, 1.0],
    ));
    let playback = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                paused: true,
                muted: true,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    sound.resume_playback(playback).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    let resumed = sound.playback_status(playback).unwrap();
    assert!(!resumed.paused);
    assert!(resumed.muted);
    assert_eq!(resumed.cursor_frame, 1);

    sound.pause_playback(playback).unwrap();
    let paused = sound.playback_status(playback).unwrap();
    assert!(paused.paused);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert_eq!(
        sound.playback_status(playback).unwrap().cursor_frame,
        paused.cursor_frame
    );

    sound.resume_playback(playback).unwrap();
    assert!(!sound.playback_status(playback).unwrap().paused);
    sound.toggle_playback(playback).unwrap();
    assert!(sound.playback_status(playback).unwrap().paused);
    sound.toggle_playback(playback).unwrap();
    assert!(!sound.playback_status(playback).unwrap().paused);
}
