#[derive(Clone, Debug, PartialEq)]
pub struct SoundConfig {
    pub enabled: bool,
    pub backend: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub master_gain: f32,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "software-mixer".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            master_gain: 1.0,
        }
    }
}
