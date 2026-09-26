use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// 当前仍活动的触点样本；结束和取消事件从活动集合移除，须从事件流读取其终止边沿。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    pub id: u64,
    pub position: [f32; 2],
    pub phase: TouchPhase,
}
