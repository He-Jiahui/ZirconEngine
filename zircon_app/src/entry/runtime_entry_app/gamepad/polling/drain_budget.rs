//! 一次 App 帧内手柄事件排空的公平性预算。
//! 计数或时间到限即让出主线程；即使开始时已超时，也允许至少一个事件前进。

use std::time::{Duration, Instant};

const GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME: usize = 256;
const GAMEPAD_DRAIN_MAX_TIME_PER_FRAME: Duration = Duration::from_millis(2);

/// 仅覆盖一次排空批次；耗尽表示需要后续批次，不表示事件队列已经为空。
pub(super) struct GamepadDrainBudget {
    processed_events: usize,
    deadline: Instant,
}

impl GamepadDrainBudget {
    pub(super) fn begin(now: Instant) -> Self {
        Self {
            processed_events: 0,
            deadline: now
                .checked_add(GAMEPAD_DRAIN_MAX_TIME_PER_FRAME)
                .unwrap_or(now),
        }
    }

    pub(super) fn record_event(&mut self) {
        self.processed_events = self.processed_events.saturating_add(1);
    }

    pub(super) fn needs_continuation(&self, now: Instant) -> bool {
        self.processed_events >= GAMEPAD_DRAIN_MAX_EVENTS_PER_FRAME
            || (self.processed_events > 0 && now >= self.deadline)
    }
}

#[cfg(test)]
#[path = "tests/drain_budget.rs"]
mod tests;
