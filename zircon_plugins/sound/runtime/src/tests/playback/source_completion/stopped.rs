use super::super::super::*;

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
            SoundExternalSourceBlock::new(10, SoundChannelLayout::mono(), vec![0.25]),
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
