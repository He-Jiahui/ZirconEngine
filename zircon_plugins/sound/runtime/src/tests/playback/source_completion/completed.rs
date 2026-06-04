use super::super::super::*;

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
