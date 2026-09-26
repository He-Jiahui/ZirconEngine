use serde::{Deserialize, Serialize};

/// 描述当前帧的即时事件队列；连续光标移动和鼠标位移可合并，合并计数在下一帧开始时归零。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEventQueueStatus {
    pub retained_events: u32,
    pub coalesced_events: u64,
}
