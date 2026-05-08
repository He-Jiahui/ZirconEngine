use serde::{Deserialize, Serialize};

use super::{AsyncTaskHandle, TaskCancellationPolicy, TaskPoolKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsyncTaskDescriptor {
    pub handle: AsyncTaskHandle,
    pub pool: TaskPoolKind,
    pub label: String,
    pub cancellation_policy: TaskCancellationPolicy,
}

impl AsyncTaskDescriptor {
    pub fn new(handle: AsyncTaskHandle, pool: TaskPoolKind, label: impl Into<String>) -> Self {
        Self {
            handle,
            pool,
            label: label.into(),
            cancellation_policy: TaskCancellationPolicy::default(),
        }
    }

    pub fn with_cancellation_policy(mut self, policy: TaskCancellationPolicy) -> Self {
        self.cancellation_policy = policy;
        self
    }
}
