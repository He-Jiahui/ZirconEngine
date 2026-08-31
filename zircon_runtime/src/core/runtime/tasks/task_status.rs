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
mod tests {
    use super::*;

    #[test]
    fn runtime_task_status_has_one_terminal_state_and_no_poll_clock() {
        let mut status = TaskStatus::pending(TaskId::new(42));
        assert_eq!(status.state, TaskState::Pending);
        assert!(!status.is_terminal());

        status.mark_running();
        assert_eq!(status.state, TaskState::Running);

        status.mark_failed("worker panicked");
        assert_eq!(status.state, TaskState::Failed);
        assert_eq!(status.failure_message.as_deref(), Some("worker panicked"));
        assert!(status.is_terminal());
    }
}
