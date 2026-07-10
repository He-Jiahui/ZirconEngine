use super::super::super::*;

#[test]
fn external_input_resamples_to_mixer_rate() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("synth.low-rate");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(24_000, AudioChannelLayout::mono(), vec![0.5, 1.0]),
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    assert_samples_near(
        &sound.render_mix(4).unwrap().samples,
        &[0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0, 1.0],
    );
}
