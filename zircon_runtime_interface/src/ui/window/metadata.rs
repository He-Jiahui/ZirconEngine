use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiInputSequence, UiInputTimestamp, UiWindowId};

/// 平台适配器与 UI 输入分发器之间共享的窗口事件上下文。
/// 转成输入事件时需保留窗口身份、时间顺序和 synthetic 标记，
/// 使路由与诊断能够识别事件归属和来源。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowEventMetadata {
    /// 事件所属窗口；身份分配与空值策略由来源决定。
    pub window_id: UiWindowId,
    /// 来源提供的单调时钟微秒值；此 DTO 不读取时钟。
    pub timestamp: UiInputTimestamp,
    /// 来源提供的顺序号；构造、复制和规范化不自动递增。
    pub sequence: UiInputSequence,
    /// 是否由来源标记为合成事件，不由事件种类推断。
    pub synthetic: bool,
}

impl UiWindowEventMetadata {
    /// 平台事件默认为非合成事件；转发已有来源标记时须再调用 synthetic。
    /// 参数按值保存；不校验身份非空或时间、序号的单调性。
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

    /// 覆盖来源标记并返回元数据，不改变时间或顺序号。
    pub const fn synthetic(mut self, synthetic: bool) -> Self {
        self.synthetic = synthetic;
        self
    }
}
