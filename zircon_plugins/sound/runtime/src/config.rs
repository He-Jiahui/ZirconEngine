use zircon_runtime::core::framework::sound::{
    SoundConvolutionBudget, SoundPluginOptions, SoundRayTracingQuality,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SoundConfig {
    pub enabled: bool,
    pub backend: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub master_gain: f32,
    pub block_size_frames: usize,
    pub max_voices: usize,
    pub max_tracks: usize,
    pub default_spatial_scale: f32,
    pub hrtf_enabled: bool,
    pub hrtf_profile: String,
    pub convolution_enabled: bool,
    pub convolution_budget: SoundConvolutionBudget,
    pub ray_tracing_quality: SoundRayTracingQuality,
    pub default_mixer_preset: String,
    pub timeline_integration: bool,
    pub dynamic_events_enabled: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self::from_plugin_options(SoundPluginOptions::default())
    }
}

impl SoundConfig {
    pub fn from_plugin_options(options: SoundPluginOptions) -> Self {
        Self {
            enabled: options.enabled,
            backend: options.backend,
            sample_rate_hz: options.sample_rate_hz,
            channel_count: options.channel_count,
            master_gain: options.global_volume_gain,
            block_size_frames: options.block_size_frames,
            max_voices: options.max_voices,
            max_tracks: options.max_tracks,
            default_spatial_scale: options.default_spatial_scale,
            hrtf_enabled: options.hrtf_enabled,
            hrtf_profile: options.hrtf_profile,
            convolution_enabled: options.convolution_enabled,
            convolution_budget: options.convolution_budget,
            ray_tracing_quality: options.ray_tracing_quality,
            default_mixer_preset: options.default_mixer_preset,
            timeline_integration: options.timeline_integration,
            dynamic_events_enabled: options.dynamic_events_enabled,
        }
    }
}

impl From<SoundPluginOptions> for SoundConfig {
    fn from(options: SoundPluginOptions) -> Self {
        Self::from_plugin_options(options)
    }
}
