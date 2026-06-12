use super::super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_runtime_options() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report
        .extensions
        .plugin_options()
        .iter()
        .any(|option| option.key == "sound.ray_tracing_quality"));
    for option_key in [
        "sound.backend",
        "sound.sample_rate_hz",
        "sound.channel_count",
        "sound.channel_layout",
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
}
