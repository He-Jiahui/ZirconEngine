use serde::{Deserialize, Serialize};

pub const DEFAULT_INPUT_EVENT_RECORDING_CAPACITY: u32 = 8_192;

/// 控制独立于帧事件队列的诊断录制；默认关闭，启用后容量限制保留的原始事件记录。
/// 容量为零仍统计丢弃数，重新启用会开始新的序列。
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
