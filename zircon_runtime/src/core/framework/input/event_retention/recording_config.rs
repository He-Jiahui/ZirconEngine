use serde::{Deserialize, Serialize};

pub const DEFAULT_INPUT_EVENT_RECORDING_CAPACITY: u32 = 8_192;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventRecordingConfig {
    pub enabled: bool,
    pub capacity: u32,
}

impl InputEventRecordingConfig {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            capacity: DEFAULT_INPUT_EVENT_RECORDING_CAPACITY,
        }
    }

    pub const fn enabled(capacity: u32) -> Self {
        Self {
            enabled: true,
            capacity,
        }
    }
}

impl Default for InputEventRecordingConfig {
    fn default() -> Self {
        Self::disabled()
    }
}
