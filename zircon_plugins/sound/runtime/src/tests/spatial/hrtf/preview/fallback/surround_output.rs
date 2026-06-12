use super::super::super::super::super::*;

#[test]
fn preview_hrtf_fallback_clears_non_binaural_surround_output() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.hrtf.preview.5_1"),
            backend: "software-test".to_string(),
            display_name: "Preview HRTF Surround Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/hrtf-preview-surround.wav",
        48_000,
        6,
        &[1.0, 2.0, 3.0, 9.0, 5.0, 6.0],
    ));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("missing-surround-profile".to_string());
    listener.left_ear_offset = [-0.08, 0.0, 0.0];
    listener.right_ear_offset = [0.08, 0.0, 0.0];
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [0.5, 0.0, 1.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[1.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
}
