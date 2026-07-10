use super::super::super::*;

#[test]
fn external_audio_source_block_reports_completed_external_source() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("particles.wind-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(48_000, AudioChannelLayout::mono(), vec![0.25, 0.5]),
        )
        .unwrap();
    let source_id = sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle.clone()),
            gain: 0.5,
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    sound.render_mix(2).unwrap();
    sound.render_mix(1).unwrap();

    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, source_id);
    assert_eq!(finished[0].input, SoundSourceInput::External(handle));
    assert_eq!(finished[0].clip, None);
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Completed);
}
