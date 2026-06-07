use super::super::super::*;

#[test]
fn cleared_external_audio_source_renders_silence_for_missing_blocks() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");

    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(48_000, SoundChannelLayout::mono(), vec![0.75]),
        )
        .unwrap();
    sound.clear_external_source(&handle).unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[0.0, 0.0]);
}
