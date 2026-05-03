use serde::{Deserialize, Serialize};

use super::{SoundMixBlock, SoundOutputDeviceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCapability {
    pub backend: String,
    pub display_name: String,
    pub realtime_capable: bool,
    pub deterministic: bool,
    pub min_sample_rate_hz: u32,
    pub max_sample_rate_hz: u32,
    pub min_channel_count: u16,
    pub max_channel_count: u16,
    pub min_block_size_frames: usize,
    pub max_block_size_frames: usize,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCallbackReport {
    pub device: SoundOutputDeviceId,
    pub backend: String,
    pub sequence_index: u64,
    pub requested_frames: usize,
    pub rendered_frames: usize,
    pub sample_count: usize,
    pub underrun: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCallbackBlock {
    pub report: SoundBackendCallbackReport,
    pub block: SoundMixBlock,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundOutputDeviceDescriptor {
    pub id: SoundOutputDeviceId,
    pub backend: String,
    pub display_name: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub block_size_frames: usize,
    pub latency_blocks: usize,
}

impl SoundOutputDeviceDescriptor {
    pub fn software(
        backend: impl Into<String>,
        sample_rate_hz: u32,
        channel_count: u16,
        block_size_frames: usize,
    ) -> Self {
        Self {
            id: SoundOutputDeviceId::default_system(),
            backend: backend.into(),
            display_name: "Software Output".to_string(),
            sample_rate_hz,
            channel_count,
            block_size_frames,
            latency_blocks: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundOutputDeviceState {
    Stopped,
    Started,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundOutputDeviceStatus {
    pub descriptor: SoundOutputDeviceDescriptor,
    pub state: SoundOutputDeviceState,
    pub rendered_blocks: u64,
    pub rendered_frames: u64,
    pub callback_count: u64,
    pub last_callback_sequence: Option<u64>,
    pub underrun_count: u64,
    pub last_error: Option<String>,
}
