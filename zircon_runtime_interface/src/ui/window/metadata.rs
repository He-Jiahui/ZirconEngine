use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiInputSequence, UiInputTimestamp, UiWindowId};

/// 平台适配器与 UI 输入分发器之间共享的窗口事件上下文。
/// 转成输入事件时需保留窗口身份、时间顺序和 synthetic 标记，
/// 使路由与诊断能够识别事件归属和来源。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowEventMetadata {
    pub window_id: UiWindowId,
    pub timestamp: UiInputTimestamp,
    pub sequence: UiInputSequence,
    pub synthetic: bool,
}

impl UiWindowEventMetadata {
    /// 平台事件默认为非合成事件；转发已有来源标记时须再调用 synthetic。
    pub const fn for_window(
        window_id: UiWindowId,
        timestamp: UiInputTimestamp,
        sequence: UiInputSequence,
    ) -> Self {
        Self {
            window_id,
            timestamp,
            sequence,
            synthetic: false,
        }
    }

    pub const fn synthetic(mut self, synthetic: bool) -> Self {
        self.synthetic = synthetic;
        self
    }
}
