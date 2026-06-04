use super::super::super::*;
use super::support::cinematic_options;

#[test]
fn sound_config_preserves_neutral_plugin_option_values() {
    let config = SoundConfig::from_plugin_options(cinematic_options());

    assert_eq!(config.backend, "software-null");
    assert_eq!(config.sample_rate_hz, 44_100);
    assert_eq!(config.channel_count, 6);
    assert_eq!(config.channel_layout, SoundChannelLayout::surround_5_1());
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
}
