use super::super::super::super::*;

#[test]
fn external_surround_block_downmixes_to_stereo_without_lfe() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.surround-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 6,
                channel_layout: SoundChannelLayout::surround_5_1(),
                samples: vec![0.10, 0.20, 0.30, 9.0, 0.40, 0.50],
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
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}
