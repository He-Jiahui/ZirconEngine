use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundBackendState {
    Ready,
    Disabled,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundBackendStatus {
    pub requested_backend: String,
    pub active_backend: Option<String>,
    pub state: SoundBackendState,
    pub detail: Option<String>,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
}
