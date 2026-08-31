use serde::{Deserialize, Serialize};

use super::{TaskCancellationPolicy, TaskId, TaskPoolKind};

/// Immutable admission contract for one runtime-owned task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub id: TaskId,
    /// Logical workload class for the shared TaskGraph scheduler. This is not
    /// a physical pool selector.
    pub kind: TaskPoolKind,
    pub label: String,
    pub cancellation_policy: TaskCancellationPolicy,
}

impl TaskDescriptor {
    pub fn new(id: TaskId, kind: TaskPoolKind, label: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            label: label.into(),
            cancellation_policy: TaskCancellationPolicy::default(),
        }
    }

    pub fn with_cancellation_policy(mut self, policy: TaskCancellationPolicy) -> Self {
        self.cancellation_policy = policy;
        self
    }
}
