use super::super::super::*;

#[test]
fn playback_gain_and_mute_controls_affect_render_and_status() {
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
    sound.unmute_playback(playback).unwrap();
    sound.set_playback_gain(playback, 0.5).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.25, 0.25]);
    let advanced = sound.playback_status(playback).unwrap();
    assert_eq!(advanced.clip, clip);
    assert_eq!(advanced.gain, 0.5);
    assert_eq!(advanced.cursor_frame, 2);

    sound.mute_playback(playback).unwrap();
    assert!(sound.playback_status(playback).unwrap().muted);
    sound.unmute_playback(playback).unwrap();
    sound.toggle_mute_playback(playback).unwrap();
    assert!(sound.playback_status(playback).unwrap().muted);
    sound.toggle_mute_playback(playback).unwrap();
    assert!(!sound.playback_status(playback).unwrap().muted);
}
