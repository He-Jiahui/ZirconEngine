use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPluginOptions {
    pub enabled: bool,
    pub backend: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub block_size_frames: usize,
    pub max_voices: usize,
    pub max_tracks: usize,
    pub hrtf_enabled: bool,
    pub hrtf_profile: String,
    pub convolution_enabled: bool,
    pub convolution_budget: SoundConvolutionBudget,
    pub ray_tracing_quality: SoundRayTracingQuality,
    pub default_mixer_preset: String,
    pub timeline_integration: bool,
    pub dynamic_events_enabled: bool,
}

impl Default for SoundPluginOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "software-mixer".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 256,
            max_voices: 128,
            max_tracks: 64,
            hrtf_enabled: false,
            hrtf_profile: "default".to_string(),
            convolution_enabled: true,
            convolution_budget: SoundConvolutionBudget::default(),
            ray_tracing_quality: SoundRayTracingQuality::Disabled,
            default_mixer_preset: "sound://mixer/default".to_string(),
            timeline_integration: true,
            dynamic_events_enabled: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundConvolutionBudget {
    pub max_impulse_responses: usize,
    pub max_partition_frames: usize,
    pub rays_per_update: usize,
}

impl Default for SoundConvolutionBudget {
    fn default() -> Self {
        Self {
            max_impulse_responses: 32,
            max_partition_frames: 1024,
            rays_per_update: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundRayTracingQuality {
    Disabled,
    Preview,
    Balanced,
    Cinematic,
}
