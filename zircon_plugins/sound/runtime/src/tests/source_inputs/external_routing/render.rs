use super::super::super::*;

#[test]
fn external_audio_source_block_renders_external_component_audio() {
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

    let first_mix = sound.render_mix(2).unwrap();

    assert_eq!(source_id.raw(), 1);
    assert_samples_near(&first_mix.samples, &[0.125, 0.125, 0.25, 0.25]);
}
