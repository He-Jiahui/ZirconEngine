use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{LaneInner, LaneState};
use crate::core::runtime::tasks::bounded_keyed_io::{
    BoundedKeyedIoDiagnostics, BoundedKeyedIoShutdownReport,
};
use crate::core::runtime::tasks::JobHandle;

/// 关闭后的可查询所有权凭证；超时等待后仍可查看积压并重试等待。
/// 丢弃 guard 会无期限等待通道内的工作和任务句柄，调用方需先解除可能阻塞的外部依赖。
pub struct BoundedKeyedIoShutdownGuard {
    pub(super) lane: Arc<LaneInner>,
}

impl BoundedKeyedIoShutdownGuard {
    pub fn is_complete(&self) -> bool {
        shutdown_complete(&self.lane.lock())
    }

    /// Waits for every shutdown-pinned entry and its worker handle to finish.
    pub fn wait(&self) {
        let mut state = self.lane.lock();
        while !shutdown_complete(&state) {
            state = self
                .lane
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        let handles = state.active_handles.clone();
        drop(state);
        for handle in handles {
            handle.wait();
        }
    }

    /// 在统一截止时间前等待通道静止；返回 false 时 guard 仍保有清理权威。
    pub fn wait_until(&self, deadline: Instant) -> bool {
        let mut state = self.lane.lock();
        while !shutdown_complete(&state) {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            state = self
                .lane
                .changed
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
        true
    }

    pub fn report(&self) -> BoundedKeyedIoShutdownReport {
        let state = self.lane.lock();
        BoundedKeyedIoShutdownReport {
            complete: shutdown_complete(&state),
            incomplete_entries: state.reserved_entries,
            failed: state.failed,
            cancelled: state.cancelled,
            diagnostics: diagnostics_for_state(&state),
        }
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        self.report().diagnostics
    }
}

impl Drop for BoundedKeyedIoShutdownGuard {
    fn drop(&mut self) {
        self.wait();
    }
}

pub(super) fn diagnostics_for_state(state: &LaneState) -> BoundedKeyedIoDiagnostics {
    let now = Instant::now();
    let oldest_age = oldest_age_at(
        now,
        state
            .queue
            .iter()
            .map(|entry| entry.enqueued_at)
            .chain(state.suspended.values().map(|entry| entry.enqueued_at))
            .chain(state.active.iter().map(|entry| entry.enqueued_at)),
    );
    BoundedKeyedIoDiagnostics {
        queue_entries: state.reserved_entries,
        retained_bytes: state.retained_bytes,
        in_flight: state.in_flight,
        oldest_age,
        submitted: state.submitted,
        completed: state.completed,
        failed: state.failed,
        cancelled: state.cancelled,
        superseded: state.superseded,
        coalesced: state.coalesced,
        worker_wall: state.worker_wall,
    }
}

fn oldest_age_at(now: Instant, enqueued_at: impl Iterator<Item = Instant>) -> Duration {
    enqueued_at
        .map(|enqueued_at| now.saturating_duration_since(enqueued_at))
        .max()
        .unwrap_or(Duration::ZERO)
}

// 静止判定同时清点预留、挂起、排队、执行及泵句柄；队列暂空本身不足以证明关闭完成。
fn shutdown_complete(state: &LaneState) -> bool {
    state.reserved_entries == 0
        && state.in_flight == 0
        && state.suspended.is_empty()
        && state.queue.is_empty()
        && state.active.is_none()
        && !state.pump_active
        && state.active_handles.iter().all(JobHandle::is_complete)
}

#[cfg(test)]
#[path = "tests/shutdown.rs"]
mod tests;
