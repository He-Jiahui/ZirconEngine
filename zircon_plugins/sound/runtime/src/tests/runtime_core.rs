use super::*;

#[test]
fn sound_plugin_registration_contributes_runtime_module_components_options_and_events() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == SOUND_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
    for component in [
        AUDIO_SOURCE_COMPONENT_TYPE,
        AUDIO_LISTENER_COMPONENT_TYPE,
        AUDIO_VOLUME_COMPONENT_TYPE,
    ] {
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|descriptor| descriptor.type_id == component));
        assert!(report
            .package_manifest
            .components
            .iter()
            .any(|descriptor| descriptor.type_id == component));
    }
    assert!(report
        .extensions
        .plugin_options()
        .iter()
        .any(|option| option.key == "sound.ray_tracing_quality"));
    for option_key in [
        "sound.backend",
        "sound.sample_rate_hz",
        "sound.channel_count",
        "sound.global_volume_gain",
        "sound.default_spatial_scale",
        "sound.block_size_frames",
        "sound.max_voices",
        "sound.max_tracks",
        "sound.hrtf_enabled",
        "sound.hrtf_profile",
        "sound.convolution_enabled",
        "sound.convolution_budget.max_impulse_responses",
        "sound.convolution_budget.max_partition_frames",
        "sound.convolution_budget.rays_per_update",
        "sound.ray_tracing_quality",
        "sound.default_mixer_preset",
        "sound.timeline_integration",
        "sound.dynamic_events_enabled",
    ] {
        assert!(
            report
                .package_manifest
                .options
                .iter()
                .any(|option| option.key == option_key),
            "missing sound option {option_key}"
        );
    }
    assert!(report
        .package_manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.id == "timeline_sequence" && !dependency.required));
    assert!(report
        .extensions
        .plugin_event_catalogs()
        .iter()
        .any(|catalog| {
            catalog.namespace == SOUND_DYNAMIC_EVENT_NAMESPACE && catalog.events.is_empty()
        }));
}

#[test]
fn default_sound_manager_renders_silence_without_active_playback() {
    let sound = DefaultSoundManager::default();
    let mix = sound.render_mix(3).unwrap();

    assert_eq!(mix.sample_rate_hz, 48_000);
    assert_eq!(mix.channel_count, 2);
    assert_eq!(mix.samples, vec![0.0; 6]);
}

#[test]
fn global_volume_gain_scales_final_mix_and_rejects_invalid_values() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/global-volume.wav", &[1.0]));
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    assert_eq!(sound.global_volume_gain().unwrap(), 1.0);
    sound.set_global_volume_gain(0.25).unwrap();
    let mix = sound.render_mix(1).unwrap();

    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);
    assert_eq!(mix.samples, vec![0.25, 0.25]);
    assert!(sound.set_global_volume_gain(f32::NAN).is_err());
    assert!(sound.set_global_volume_gain(-0.1).is_err());
}

#[test]
fn sound_config_preserves_neutral_plugin_option_values() {
    let options = SoundPluginOptions {
        enabled: true,
        backend: "software-null".to_string(),
        sample_rate_hz: 44_100,
        channel_count: 6,
        global_volume_gain: 0.25,
        block_size_frames: 128,
        max_voices: 32,
        max_tracks: 12,
        default_spatial_scale: 2.5,
        hrtf_enabled: true,
        hrtf_profile: "cinematic-room".to_string(),
        convolution_enabled: false,
        convolution_budget: SoundConvolutionBudget {
            max_impulse_responses: 7,
            max_partition_frames: 512,
            rays_per_update: 64,
        },
        ray_tracing_quality: SoundRayTracingQuality::Balanced,
        default_mixer_preset: "sound://mixer/cinematic".to_string(),
        timeline_integration: false,
        dynamic_events_enabled: false,
    };

    let config = SoundConfig::from_plugin_options(options);
    assert_eq!(config.backend, "software-null");
    assert_eq!(config.sample_rate_hz, 44_100);
    assert_eq!(config.channel_count, 6);
    assert_eq!(config.master_gain, 0.25);
    assert_eq!(config.block_size_frames, 128);
    assert_eq!(config.max_voices, 32);
    assert_eq!(config.max_tracks, 12);
    assert_eq!(config.default_spatial_scale, 2.5);
    assert!(config.hrtf_enabled);
    assert_eq!(config.hrtf_profile, "cinematic-room");
    assert!(!config.convolution_enabled);
    assert_eq!(config.convolution_budget.max_impulse_responses, 7);
    assert_eq!(config.convolution_budget.max_partition_frames, 512);
    assert_eq!(config.convolution_budget.rays_per_update, 64);
    assert_eq!(config.ray_tracing_quality, SoundRayTracingQuality::Balanced);
    assert_eq!(config.default_mixer_preset, "sound://mixer/cinematic");
    assert!(!config.timeline_integration);
    assert!(!config.dynamic_events_enabled);

    let sound = DefaultSoundManager::with_config(None, config);
    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);
    assert_eq!(sound.default_spatial_scale().unwrap(), 2.5);
    let mix = sound.render_mix(1).unwrap();
    assert_eq!(mix.sample_rate_hz, 44_100);
    assert_eq!(mix.channel_count, 6);
    assert_eq!(mix.samples, vec![0.0; 6]);
}
