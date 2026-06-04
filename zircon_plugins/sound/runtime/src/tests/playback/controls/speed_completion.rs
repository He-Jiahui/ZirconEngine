use super::super::super::*;

#[test]
fn playback_speed_controls_validate_values_and_report_completion_errors() {
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

    sound.set_playback_speed(playback, 2.0).unwrap();
    assert!(sound.set_playback_speed(playback, f32::NAN).is_err());
    assert!(sound.set_playback_speed(playback, 0.0).is_err());
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.375, 0.375]);
    assert_eq!(sound.playback_status(playback).unwrap().speed, 2.0);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(sound.playback_status(playback).is_err());
    assert!(sound.toggle_playback(playback).is_err());
    assert!(sound.toggle_mute_playback(playback).is_err());
}
