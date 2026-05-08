use serde::{Deserialize, Serialize};

use super::TaskPoolKind;

const MIN_WORKER_THREADS: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPoolDescriptor {
    pub kind: TaskPoolKind,
    pub worker_threads: Option<usize>,
    pub thread_name: String,
}

impl TaskPoolDescriptor {
    pub fn new(kind: TaskPoolKind) -> Self {
        Self {
            kind,
            worker_threads: None,
            thread_name: kind.default_thread_name().to_string(),
        }
    }

    pub fn compute() -> Self {
        Self::new(TaskPoolKind::Compute)
    }

    pub fn async_compute() -> Self {
        Self::new(TaskPoolKind::AsyncCompute)
    }

    pub fn io() -> Self {
        Self::new(TaskPoolKind::Io)
    }

    pub fn with_worker_threads(mut self, worker_threads: usize) -> Self {
        self.worker_threads = Some(worker_threads.max(MIN_WORKER_THREADS));
        self
    }

    pub fn with_thread_name(mut self, thread_name: impl Into<String>) -> Self {
        self.thread_name = thread_name.into();
        self
    }
}

impl Default for TaskPoolDescriptor {
    fn default() -> Self {
        Self::compute()
    }
}
