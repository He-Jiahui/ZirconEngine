use super::super::super::*;

#[test]
fn external_audio_source_block_reports_empty_after_block_is_consumed() {
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
            input: SoundSourceInput::External(handle),
            gain: 0.5,
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    sound.render_mix(2).unwrap();
    let second_mix = sound.render_mix(1).unwrap();

    assert_samples_near(&second_mix.samples, &[0.0, 0.0]);
    assert!(sound.source_empty(source_id).unwrap());
}
