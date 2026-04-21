use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundMixBlock {
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub samples: Vec<f32>,
}

impl SoundMixBlock {
    pub fn silent(sample_rate_hz: u32, channel_count: u16, frames: usize) -> Self {
        Self {
            sample_rate_hz,
            channel_count,
            samples: vec![0.0; frames.saturating_mul(channel_count as usize)],
        }
    }
}
