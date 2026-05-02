use super::SoundTrackId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundClipInfo {
    pub locator: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub frame_count: usize,
    pub duration_seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPlaybackSettings {
    pub gain: f32,
    pub looped: bool,
    pub output_track: SoundTrackId,
    pub pan: f32,
}

impl Default for SoundPlaybackSettings {
    fn default() -> Self {
        Self {
            gain: 1.0,
            looped: false,
            output_track: SoundTrackId::master(),
            pan: 0.0,
        }
    }
}
