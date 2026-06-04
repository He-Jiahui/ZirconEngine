use super::super::*;

#[test]
fn playback_completion_events_track_empty_and_stopped_sinks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/playback-empty.wav", &[0.5]));
    let playback = sound
        .play_clip(clip, SoundPlaybackSettings::REMOVE)
        .unwrap();

    assert!(!sound.playback_empty(playback).unwrap());
    assert!(sound.drain_finished_playbacks().unwrap().is_empty());
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.5, 0.5]);
    assert_eq!(sound.playback_status(playback).unwrap().cursor_frame, 1);
    assert!(!sound.playback_empty(playback).unwrap());
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(sound.playback_status(playback).is_err());
    assert!(sound.playback_empty(playback).unwrap());
    let finished = sound.drain_finished_playbacks().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].playback, playback);
    assert_eq!(finished[0].clip, clip);
    assert_eq!(finished[0].reason, SoundPlaybackFinishReason::Completed);
    assert_eq!(
        finished[0].completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );
    assert_eq!(finished[0].output_track, SoundTrackId::master());
    assert!(sound.drain_finished_playbacks().unwrap().is_empty());
    assert!(sound.playback_empty(playback).is_err());

    let stopped = sound
        .play_clip(clip, SoundPlaybackSettings::DESPAWN)
        .unwrap();
    assert!(!sound.playback_empty(stopped).unwrap());
    sound.stop_playback(stopped).unwrap();
    assert!(sound.playback_status(stopped).is_err());
    assert!(sound.playback_empty(stopped).unwrap());
    let finished = sound.drain_finished_playbacks().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].playback, stopped);
    assert_eq!(finished[0].reason, SoundPlaybackFinishReason::Stopped);
    assert_eq!(
        finished[0].completion_action,
        SoundPlaybackCompletionAction::DespawnEntity
    );
    assert!(sound.stop_playback(stopped).is_err());
    assert!(sound.playback_empty(stopped).is_err());
}
