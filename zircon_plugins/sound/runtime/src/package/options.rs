use zircon_runtime::plugin::PluginOptionManifest;

pub fn sound_options() -> Vec<PluginOptionManifest> {
    vec![
        PluginOptionManifest::new("sound.backend", "Audio Backend", "string", "software-mixer"),
        PluginOptionManifest::new("sound.sample_rate_hz", "Sample Rate", "integer", "48000"),
        PluginOptionManifest::new("sound.channel_count", "Channel Count", "integer", "2"),
        PluginOptionManifest::new("sound.global_volume_gain", "Global Volume", "number", "1.0"),
        PluginOptionManifest::new(
            "sound.default_spatial_scale",
            "Default Spatial Scale",
            "number",
            "1.0",
        ),
        PluginOptionManifest::new("sound.block_size_frames", "Block Size", "integer", "256"),
        PluginOptionManifest::new("sound.max_voices", "Max Voices", "integer", "128"),
        PluginOptionManifest::new("sound.max_tracks", "Max Tracks", "integer", "64"),
        PluginOptionManifest::new("sound.hrtf_enabled", "HRTF", "bool", "false"),
        PluginOptionManifest::new("sound.hrtf_profile", "HRTF Profile", "string", "default"),
        PluginOptionManifest::new(
            "sound.convolution_enabled",
            "Convolution Reverb",
            "bool",
            "true",
        ),
        PluginOptionManifest::new(
            "sound.convolution_budget.max_impulse_responses",
            "Max Impulse Responses",
            "integer",
            "32",
        ),
        PluginOptionManifest::new(
            "sound.convolution_budget.max_partition_frames",
            "Max Convolution Partition Frames",
            "integer",
            "1024",
        ),
        PluginOptionManifest::new(
            "sound.convolution_budget.rays_per_update",
            "Convolution Rays Per Update",
            "integer",
            "0",
        )
        .with_required_capability("runtime.capability.ray_query"),
        PluginOptionManifest::new(
            "sound.ray_tracing_quality",
            "Ray Tracing Quality",
            "enum",
            "disabled",
        )
        .with_required_capability("runtime.capability.ray_query"),
        PluginOptionManifest::new(
            "sound.default_mixer_preset",
            "Default Mixer Preset",
            "string",
            "sound://mixer/default",
        ),
        PluginOptionManifest::new(
            "sound.timeline_integration",
            "Timeline Automation",
            "bool",
            "true",
        )
        .with_required_capability("editor.extension.timeline_sequence_authoring"),
        PluginOptionManifest::new(
            "sound.dynamic_events_enabled",
            "Dynamic Events",
            "bool",
            "true",
        ),
    ]
}
