use serde::{Deserialize, Serialize};

use super::{AsyncTaskHandle, AsyncTaskState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsyncTaskStatus {
    pub handle: AsyncTaskHandle,
    pub state: AsyncTaskState,
    pub poll_count: u32,
    pub failure_message: Option<String>,
}

impl AsyncTaskStatus {
    pub fn pending(handle: AsyncTaskHandle) -> Self {
        Self {
            handle,
            state: AsyncTaskState::Pending,
            poll_count: 0,
            failure_message: None,
        }
    }

    pub const fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn mark_running(&mut self) {
        self.state = AsyncTaskState::Running;
        self.failure_message = None;
    }

    pub fn mark_completed(&mut self) {
        self.state = AsyncTaskState::Completed;
        self.failure_message = None;
    }

    pub fn mark_failed(&mut self, message: impl Into<String>) {
        self.state = AsyncTaskState::Failed;
        self.failure_message = Some(message.into());
    }

    pub fn mark_cancelled(&mut self) {
        self.state = AsyncTaskState::Cancelled;
        self.failure_message = None;
    }

    pub fn record_poll(&mut self) {
        self.poll_count = self.poll_count.saturating_add(1);
    }
}
