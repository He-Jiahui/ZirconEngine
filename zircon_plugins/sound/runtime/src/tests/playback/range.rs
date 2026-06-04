use super::super::*;

#[test]
fn playback_start_duration_seek_and_loop_range_match_sink_position_controls() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/playback-range.wav",
        &[0.1, 0.2, 0.3, 0.4, 0.5],
    ));
    let frame_seconds = 1.0 / 48_000.0;
    let playback = sound
        .play_clip(
            clip,
            SoundPlaybackSettings::LOOP
                .with_start_seconds(frame_seconds * 2.0)
                .with_duration_seconds(frame_seconds * 2.0),
        )
        .unwrap();

    let initial = sound.playback_status(playback).unwrap();
    assert_eq!(initial.range_start_frame, 2);
    assert_eq!(initial.range_end_frame, Some(4));
    assert_eq!(initial.cursor_frame, 2);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.3, 0.3]);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.4, 0.4]);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.3, 0.3]);

    sound
        .seek_playback_seconds(playback, frame_seconds * 3.0)
        .unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.4, 0.4]);
    sound.seek_playback_seconds(playback, 0.0).unwrap();
    assert_eq!(sound.playback_status(playback).unwrap().cursor_frame, 2);
    sound.seek_playback_seconds(playback, 1.0).unwrap();
    assert_eq!(sound.playback_status(playback).unwrap().cursor_frame, 4);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.3, 0.3]);

    assert!(sound.seek_playback_seconds(playback, f32::NAN).is_err());
    assert!(sound
        .seek_playback_seconds(playback, -frame_seconds)
        .is_err());
    assert!(sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                duration_seconds: Some(0.0),
                ..SoundPlaybackSettings::default()
            },
        )
        .is_err());
}
