use super::super::super::*;

#[test]
fn playback_initial_status_reflects_paused_muted_settings() {
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

    let initial = sound.playback_status(playback).unwrap();
    assert!(initial.paused);
    assert!(initial.muted);
    assert_eq!(initial.gain, 1.0);
    assert_eq!(initial.speed, 1.0);
    assert_eq!(
        initial.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert_eq!(sound.playback_status(playback).unwrap().cursor_frame, 0);
}
