use super::super::super::super::*;

#[test]
fn external_discrete_4_block_folds_overflow_pair_into_stereo_output() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.discrete-4-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(
                48_000,
                AudioChannelLayout::discrete(4),
                vec![0.10, 0.20, 0.40, 0.50],
            ),
        )
        .unwrap();
    sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.30, 0.45]);
}
