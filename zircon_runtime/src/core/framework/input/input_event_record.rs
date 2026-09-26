use serde::{Deserialize, Serialize};

use super::InputEvent;

/// 可选录制中的原始事件与采集顺序；帧队列合并不会改写记录，回放按 `event` 重新提交。
/// 序号只在一次启用周期内单调，时间戳用于诊断而非回放调度。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEventRecord {
    pub sequence: u64,
    pub timestamp_millis: u64,
    pub event: InputEvent,
}
