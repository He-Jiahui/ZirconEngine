use super::super::super::super::super::*;
use super::super::super::super::support::test_hrtf_profile;

#[test]
fn loaded_hrtf_profile_clears_non_binaural_surround_output() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.hrtf.loaded.5_1"),
            backend: "software-test".to_string(),
            display_name: "Loaded HRTF Surround Test Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 6,
            channel_layout: SoundChannelLayout::surround_5_1(),
            block_size_frames: 1,
            latency_blocks: 1,
        })
        .unwrap();
    sound
        .load_hrtf_profile(test_hrtf_profile("loaded-surround"))
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_channels(
        "res://sound/hrtf-loaded-surround.wav",
        48_000,
        6,
        &[1.0, 2.0, 3.0, 9.0, 5.0, 6.0],
    ));
    let mut listener = test_listener();
    listener.hrtf_profile = Some("loaded-surround".to_string());
    sound.update_listener(listener).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        attenuation: SoundAttenuationMode::None,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_samples_near(&mix.samples, &[0.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
}
