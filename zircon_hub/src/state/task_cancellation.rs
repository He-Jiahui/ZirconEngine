use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TaskCancellationToken {
    task_id: u64,
    cancelled: Arc<AtomicBool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskExecutionOutcome<T> {
    Completed(T),
    Cancelled,
}

impl TaskCancellationToken {
    pub fn new(task_id: u64) -> Self {
        debug_assert_ne!(task_id, 0, "running task ids must be non-zero");
        Self {
            task_id,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn task_id(&self) -> u64 {
        self.task_id
    }

    pub fn request_cancellation(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancellation_requested(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[cfg(test)]
#[path = "tests/task_cancellation.rs"]
mod tests;
