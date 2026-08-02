use std::fmt;

use super::{Command, CommandQueue};

/// A per-system deferred-command buffer produced outside the World write window.
///
/// The compiled schedule owns the `(system_order, system_id)` key. Worker completion
/// order is intentionally irrelevant: `CommandQueue::merge_worker_buffers` sorts by this
/// key before moving commands into the World-owned queue.
#[derive(Debug)]
pub struct WorkerCommandBuffer {
    system_order: i32,
    system_id: String,
    queue: CommandQueue,
}

impl WorkerCommandBuffer {
    pub fn with_capacity(
        system_order: i32,
        system_id: impl Into<String>,
        command_capacity: usize,
    ) -> Self {
        Self {
            system_order,
            system_id: system_id.into(),
            queue: CommandQueue::with_capacity(command_capacity),
        }
    }

    pub fn push<C>(&mut self, command: C)
    where
        C: Command,
    {
        self.queue.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn merge_into(&mut self, destination: &mut CommandQueue) {
        destination.append(&mut self.queue);
    }

    fn key(&self) -> (i32, &str) {
        (self.system_order, &self.system_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCommandBufferMergeError {
    system_order: i32,
    system_id: String,
}

impl WorkerCommandBufferMergeError {
    fn duplicate(buffer: &WorkerCommandBuffer) -> Self {
        Self {
            system_order: buffer.system_order,
            system_id: buffer.system_id.clone(),
        }
    }

    pub fn system_order(&self) -> i32 {
        self.system_order
    }

    pub fn system_id(&self) -> &str {
        &self.system_id
    }
}

impl fmt::Display for WorkerCommandBufferMergeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate worker command buffer key ({}, {})",
            self.system_order, self.system_id
        )
    }
}

impl std::error::Error for WorkerCommandBufferMergeError {}

impl CommandQueue {
    /// Moves worker-local command buffers into this World-owned queue in compiled
    /// schedule order. Duplicate schedule keys are rejected before any payload moves.
    pub fn merge_worker_buffers(
        &mut self,
        buffers: &mut [WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        buffers.sort_by(|left, right| left.key().cmp(&right.key()));
        for pair in buffers.windows(2) {
            if pair[0].key() == pair[1].key() {
                return Err(WorkerCommandBufferMergeError::duplicate(&pair[0]));
            }
        }
        for buffer in buffers {
            self.append(&mut buffer.queue);
        }
        Ok(())
    }

    pub(crate) fn merge_worker_buffer_refs(
        &mut self,
        buffers: &mut [&mut WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        buffers.sort_by(|left, right| left.key().cmp(&right.key()));
        for pair in buffers.windows(2) {
            if pair[0].key() == pair[1].key() {
                return Err(WorkerCommandBufferMergeError::duplicate(pair[0]));
            }
        }
        for buffer in buffers {
            buffer.merge_into(self);
        }
        Ok(())
    }
}
