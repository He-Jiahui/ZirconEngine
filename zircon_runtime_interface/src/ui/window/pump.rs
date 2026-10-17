use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiDispatchEffect, UiInputEvent};

use super::{UiWindowEvent, UiWindowPlatformInputEvent};

/// 窗口通知或已经规范化的输入事件；拥有事件数据，不拥有宿主窗口或分发状态。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowInputPumpEvent {
    Window(UiWindowEvent),
    Input(UiInputEvent),
}

impl UiWindowInputPumpEvent {
    /// 仅窗口分支的显式重绘通知返回 `true`。
    pub const fn is_redraw_request(&self) -> bool {
        matches!(self, Self::Window(event) if event.is_redraw_request())
    }

    /// 只投影窗口事件的瞬态关闭命令；输入分支返回 `None`，此处不执行命令。
    pub fn transient_dismissal_effect(&self) -> Option<UiDispatchEffect> {
        match self {
            Self::Window(event) => event.transient_dismissal_effect(),
            Self::Input(_) => None,
        }
    }
}

/// 按追加顺序拥有泵事件；构造、收集和遍历均不调用 UI 分发器。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowInputPumpBatch {
    /// 待消费的有序事件；公开集合允许调用方自行编辑顺序。
    pub events: Vec<UiWindowInputPumpEvent>,
}

impl UiWindowInputPumpBatch {
    /// `capacity` 只预留初始容量，不限制可追加事件数，也不保证最终容量恰好相等。
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// 追加一个拥有的事件，不合并、不排序、不分发。
    pub fn push(&mut self, event: UiWindowInputPumpEvent) {
        self.events.push(event);
    }

    /// 只合并队尾相邻的 redraw；中间出现输入或窗口状态事件时保留后续 redraw，避免改写批次状态转移顺序。
    /// 保留已有队尾并丢弃新重绘；不比较窗口身份或原因，调用方须选择合适的合并范围。
    pub fn push_coalesced(&mut self, event: UiWindowInputPumpEvent) {
        if event.is_redraw_request()
            && self
                .events
                .last()
                .is_some_and(UiWindowInputPumpEvent::is_redraw_request)
        {
            return;
        }
        self.push(event);
    }

    /// 消费平台事件并规范化后追加为输入；不会进行重绘合并。
    pub fn push_platform_input(&mut self, event: UiWindowPlatformInputEvent) {
        self.push(UiWindowInputPumpEvent::Input(event.normalize()));
    }

    /// 惰性借用批次并按顺序产出独立命令值；迭代器存活期间保留借用，不删除或执行事件。
    pub fn transient_dismissal_effects(&self) -> impl Iterator<Item = UiDispatchEffect> + '_ {
        self.events
            .iter()
            .filter_map(UiWindowInputPumpEvent::transient_dismissal_effect)
    }
}
