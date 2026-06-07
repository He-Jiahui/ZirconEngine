use super::super::*;

#[test]
fn external_audio_source_block_routes_other_component_audio() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("particles.wind-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(48_000, SoundChannelLayout::mono(), vec![0.25, 0.5]),
        )
        .unwrap();
    let source_id = sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle.clone()),
            gain: 0.5,
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let first_mix = sound.render_mix(2).unwrap();
    let second_mix = sound.render_mix(1).unwrap();

    assert_eq!(source_id.raw(), 1);
    assert_samples_near(&first_mix.samples, &[0.125, 0.125, 0.25, 0.25]);
    assert_samples_near(&second_mix.samples, &[0.0, 0.0]);
    assert!(sound.source_empty(source_id).unwrap());
    let finished = sound.drain_finished_sources().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].source, source_id);
    assert_eq!(finished[0].input, SoundSourceInput::External(handle));
    assert_eq!(finished[0].clip, None);
    assert_eq!(finished[0].reason, SoundSourceFinishReason::Completed);
}
