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
