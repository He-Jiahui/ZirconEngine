use serde::{Deserialize, Serialize};

use super::{TaskId, TaskState};

/// Snapshot of the lifecycle authority owned by the runtime task executor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStatus {
    pub id: TaskId,
    pub state: TaskState,
    pub failure_message: Option<String>,
}

impl TaskStatus {
    pub fn pending(id: TaskId) -> Self {
        Self {
            id,
            state: TaskState::Pending,
            failure_message: None,
        }
    }

    pub const fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    // 本状态值转为 Running 时清空旧失败文本；JobHandle::task_status 仅在 Failed 状态投影 panic 说明。
    pub(crate) fn mark_running(&mut self) {
        self.state = TaskState::Running;
        self.failure_message = None;
    }

    pub(crate) fn mark_completed(&mut self) {
        self.state = TaskState::Completed;
        self.failure_message = None;
    }

    pub(crate) fn mark_failed(&mut self, message: impl Into<String>) {
        self.state = TaskState::Failed;
        self.failure_message = Some(message.into());
    }

    pub(crate) fn mark_cancelled(&mut self) {
        self.state = TaskState::Cancelled;
        self.failure_message = None;
    }
}

#[cfg(test)]
#[path = "tests/task_status.rs"]
mod tests;
