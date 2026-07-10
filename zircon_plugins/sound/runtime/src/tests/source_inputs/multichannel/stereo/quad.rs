use super::super::super::super::*;

#[test]
fn external_quad_block_downmixes_rear_pair_into_stereo_front_pair() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.quad-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 4,
                channel_layout: AudioChannelLayout::quad(),
                samples: vec![0.10, 0.20, 0.40, 0.50],
            },
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
