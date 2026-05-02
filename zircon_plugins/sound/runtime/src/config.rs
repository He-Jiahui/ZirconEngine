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
    pub hrtf_enabled: bool,
    pub convolution_enabled: bool,
    pub ray_tracing_quality: zircon_runtime::core::framework::sound::SoundRayTracingQuality,
    pub timeline_integration: bool,
    pub dynamic_events_enabled: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "software-mixer".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            master_gain: 1.0,
            block_size_frames: 256,
            max_voices: 128,
            max_tracks: 64,
            hrtf_enabled: false,
            convolution_enabled: true,
            ray_tracing_quality:
                zircon_runtime::core::framework::sound::SoundRayTracingQuality::Disabled,
            timeline_integration: true,
            dynamic_events_enabled: true,
        }
    }
}
