//! 将同一次工具状态变化的事件串与单调版本绑定，交给服务层有序投递；版本耗尽由服务进入故障状态，不能环回后继续通知。

use serde::{Deserialize, Serialize};

use super::ToolLifecycleEvent;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ToolTransitionRevision(u64);

impl ToolTransitionRevision {
    pub const INITIAL: Self = Self(0);

    pub const fn value(self) -> u64 {
        self.0
    }

    pub(crate) const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn for_test(value: u64) -> Self {
        Self(value)
    }
}

/// 一笔不可拆分的工具生命周期通知；事件顺序承载抢占、结束和晋升的因果关系，接收端应按版本处理。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolTransitionBatch {
    revision: ToolTransitionRevision,
    events: Vec<ToolLifecycleEvent>,
}

impl ToolTransitionBatch {
    pub(crate) fn new(revision: ToolTransitionRevision, events: Vec<ToolLifecycleEvent>) -> Self {
        debug_assert!(!events.is_empty());
        Self { revision, events }
    }

    pub const fn revision(&self) -> ToolTransitionRevision {
        self.revision
    }

    pub fn events(&self) -> &[ToolLifecycleEvent] {
        &self.events
    }
}
