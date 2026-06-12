use super::super::super::*;

#[test]
fn external_5_1_block_folds_center_into_quad_front_pair_without_lfe() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.external.quad"),
            backend: "software-test".to_string(),
            display_name: "External Quad Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 4,
            channel_layout: SoundChannelLayout::quad(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let handle = ExternalAudioSourceHandle::new("cinematic.5_1-center-to-quad-bed");
    sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock::new(
                48_000,
                SoundChannelLayout::surround_5_1(),
                vec![0.10, 0.20, 0.30, 9.0, 0.40, 0.50],
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
    let center = 0.30 * std::f32::consts::FRAC_1_SQRT_2;

    assert_eq!(mix.channel_layout, SoundChannelLayout::quad());
    assert_samples_near(&mix.samples, &[0.10 + center, 0.20 + center, 0.40, 0.50]);
}
