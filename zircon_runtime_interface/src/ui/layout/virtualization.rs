use serde::{Deserialize, Serialize};

/// 虚拟列表的逻辑索引半开窗口，包含 overscan 项；右端点不属于该窗口。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiVirtualListWindow {
    pub first_visible: usize,
    pub last_visible_exclusive: usize,
}
