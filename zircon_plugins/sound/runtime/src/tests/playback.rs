use super::*;

#[test]
fn source_speed_and_muted_controls_match_bevy_playback_settings() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip(
        "res://sound/source-speed-muted.wav",
        &[0.25, 0.5, 0.75],
    ));

    let mut source = SoundSourceDescriptor::clip(clip);
    source.speed = 2.0;
    source.muted = true;
    let source_id = sound.create_source(source.clone()).unwrap();

    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);

    source.id = Some(source_id);
    source.muted = false;
    sound.update_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.75, 0.75]);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.speed = 0.0;
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.speed = f32::NAN;
    assert!(sound.create_source(invalid).is_err());
}

#[test]
fn source_runtime_controls_match_bevy_audio_sink_controls() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.source_controls"),
            backend: "software-test".to_string(),
            display_name: "Source Controls Test Output".to_string(),
            sample_rate_hz: 10,
            channel_count: 2,
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/source-controls.wav",
        10,
        &[0.1, 0.2, 0.3, 0.4],
    ));
    let source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();

    sound.pause_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert_eq!(sound.source_status(source).unwrap().cursor_frame, 0);
    assert!(!sound.source_status(source).unwrap().playing);

    sound.resume_source(source).unwrap();
    sound.seek_source_seconds(source, 0.2).unwrap();
    sound.set_source_gain(source, 2.0).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.6, 0.6]);
    assert_eq!(sound.source_status(source).unwrap().cursor_frame, 3);

    sound.mute_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(sound.source_status(source).unwrap().muted);

    sound.toggle_mute_source(source).unwrap();
    sound.set_source_speed(source, 0.5).unwrap();
    let status = sound.source_status(source).unwrap();
    assert_eq!(status.speed, 0.5);
    assert!(!status.muted);

    sound.toggle_source(source).unwrap();
    assert!(!sound.source_status(source).unwrap().playing);
    sound.toggle_source(source).unwrap();
    assert!(sound.source_status(source).unwrap().playing);

    assert!(sound.seek_source_seconds(source, -0.1).is_err());
    assert!(sound.set_source_gain(source, f32::NAN).is_err());
    assert!(sound.set_source_speed(source, 0.0).is_err());
    assert!(sound.unmute_source(SoundSourceId::new(999_999)).is_err());
}

#[test]
fn source_start_and_duration_limit_clip_playback_range() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.source_range"),
            backend: "software-test".to_string(),
            display_name: "Source Range Test Output".to_string(),
            sample_rate_hz: 10,
            channel_count: 2,
            block_size_frames: 3,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/source-range.wav",
        10,
        &[0.1, 0.2, 0.3, 0.4],
    ));

    let mut source = SoundSourceDescriptor::clip(clip);
    source.looped = true;
    source.start_seconds = Some(0.1);
    source.duration_seconds = Some(0.2);
    let source_id = sound.create_source(source).unwrap();

    let status = sound.source_status(source_id).unwrap();
    assert_eq!(status.range_start_frame, 1);
    assert_eq!(status.range_end_frame, Some(3));
    assert_eq!(status.cursor_frame, 0);
    assert!(status.looped);

    assert_eq!(
        sound.render_mix(3).unwrap().samples,
        vec![0.2, 0.2, 0.3, 0.3, 0.2, 0.2]
    );
    assert_eq!(sound.source_status(source_id).unwrap().cursor_frame, 2);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.start_seconds = Some(-0.1);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.duration_seconds = Some(0.0);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.duration_seconds = Some(f32::NAN);
    assert!(sound.create_source(invalid).is_err());
}

#[test]
fn source_completion_reports_cleanup_intent() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/source-finished.wav", &[0.5]));

    let mut source = SoundSourceDescriptor::clip(clip);
    source.completion_action = SoundPlaybackCompletionAction::RemoveAudioComponents;
    let source_id = sound.create_source(source).unwrap();

    assert!(!sound.source_empty(source_id).unwrap());
    assert_eq!(
        sound.source_status(source_id).unwrap().input,
        SoundSourceInput::Clip(clip)
    );
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.5, 0.5]);
    assert!(sound.drain_finished_sources().unwrap().is_empty());

    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(sound.source_status(source_id).is_err());
    assert!(sound.source_empty(source_id).unwrap());
    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, source_id);
    assert_eq!(finished[0].input, SoundSourceInput::Clip(clip));
    assert_eq!(finished[0].clip, Some(clip));
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Completed);
    assert_eq!(
        finished[0].completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );
    assert!(sound.remove_source(source_id).is_err());
    assert!(sound.source_empty(source_id).is_err());
    assert!(sound.drain_finished_sources().unwrap().is_empty());
}

#[test]
fn stop_source_reports_cleanup_intent_for_any_input() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/source-stop.wav", &[0.5]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.completion_action = SoundPlaybackCompletionAction::DespawnEntity;
    let source_id = sound.create_source(source).unwrap();

    sound.stop_source(source_id).unwrap();
    assert!(sound.source_status(source_id).is_err());
    assert!(sound.source_empty(source_id).unwrap());
    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, source_id);
    assert_eq!(finished[0].input, SoundSourceInput::Clip(clip));
    assert_eq!(finished[0].clip, Some(clip));
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Stopped);
    assert_eq!(
        finished[0].completion_action,
        SoundPlaybackCompletionAction::DespawnEntity
    );
    assert!(sound.source_empty(source_id).is_err());

    let external = ExternalAudioSourceHandle::new("source.stop.external");
    sound
        .submit_external_source_block(
            external.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 10,
                channel_count: 1,
                samples: vec![0.25],
            },
        )
        .unwrap();
    let mut external_source = SoundSourceDescriptor::clip(clip);
    external_source.input = SoundSourceInput::External(external.clone());
    let external_id = sound.create_source(external_source).unwrap();

    sound.stop_source(external_id).unwrap();
    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, external_id);
    assert_eq!(finished[0].input, SoundSourceInput::External(external));
    assert_eq!(finished[0].clip, None);
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Stopped);
    assert!(sound.stop_source(SoundSourceId::new(999_999)).is_err());
}

#[test]
fn playback_settings_presets_match_bevy_playback_modes() {
    assert_eq!(
        SoundPlaybackSettings::default(),
        SoundPlaybackSettings::ONCE
    );
    assert!(!SoundPlaybackSettings::ONCE.looped);
    assert_eq!(
        SoundPlaybackSettings::ONCE.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert!(SoundPlaybackSettings::LOOP.looped);
    assert_eq!(
        SoundPlaybackSettings::LOOP.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert!(!SoundPlaybackSettings::DESPAWN.looped);
    assert_eq!(
        SoundPlaybackSettings::DESPAWN.completion_action,
        SoundPlaybackCompletionAction::DespawnEntity
    );
    assert!(!SoundPlaybackSettings::REMOVE.looped);
    assert_eq!(
        SoundPlaybackSettings::REMOVE.completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );

    let customized = SoundPlaybackSettings::LOOP
        .paused()
        .muted()
        .with_gain(0.5)
        .with_speed(2.0)
        .with_pan(-0.25)
        .with_start_seconds(0.1)
        .with_duration_seconds(0.2)
        .with_completion_action(SoundPlaybackCompletionAction::RemoveAudioComponents)
        .with_looped(false);
    assert!(customized.paused);
    assert!(customized.muted);
    assert_eq!(customized.gain, 0.5);
    assert_eq!(customized.speed, 2.0);
    assert_eq!(customized.pan, -0.25);
    assert_eq!(customized.start_seconds, Some(0.1));
    assert_eq!(customized.duration_seconds, Some(0.2));
    assert_eq!(
        customized.completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );
    assert!(!customized.looped);
}

#[test]
fn playback_settings_reject_non_finite_initial_mix_parameters() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/playback-invalid.wav", &[0.5]));

    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_gain(f32::NAN),)
        .is_err());
    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_gain(f32::INFINITY),)
        .is_err());
    assert!(sound
        .play_clip(clip, SoundPlaybackSettings::ONCE.with_pan(f32::NAN),)
        .is_err());
    assert!(sound
        .play_clip(
            clip,
            SoundPlaybackSettings::ONCE.with_pan(f32::NEG_INFINITY),
        )
        .is_err());
}

#[test]
fn playback_pause_resume_and_status_match_sink_lifecycle_controls() {
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

    sound.resume_playback(playback).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    let paused = sound.playback_status(playback).unwrap();
    assert!(!paused.paused);
    assert!(paused.muted);
    assert_eq!(paused.cursor_frame, 1);

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
