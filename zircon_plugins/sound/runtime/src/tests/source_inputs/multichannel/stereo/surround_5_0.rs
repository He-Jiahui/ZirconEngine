use super::super::super::super::*;

#[test]
fn external_5_0_block_uses_named_speaker_downmix_from_channel_count() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("cinematic.5_0-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 5,
                channel_layout: AudioChannelLayout::surround_5_0(),
                samples: vec![0.10, 0.20, 0.30, 0.40, 0.50],
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

    assert_eq!(mix.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&mix.samples, &[0.10 + center + 0.20, 0.20 + center + 0.25]);
}
