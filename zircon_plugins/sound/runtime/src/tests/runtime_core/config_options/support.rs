use super::super::super::*;

pub(super) fn cinematic_options() -> SoundPluginOptions {
    SoundPluginOptions {
        enabled: true,
        backend: "software-null".to_string(),
        sample_rate_hz: 44_100,
        channel_count: 6,
        channel_layout: SoundChannelLayout::surround_5_1(),
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
    }
}

pub(super) fn cinematic_config() -> SoundConfig {
    SoundConfig::from_plugin_options(cinematic_options())
}
