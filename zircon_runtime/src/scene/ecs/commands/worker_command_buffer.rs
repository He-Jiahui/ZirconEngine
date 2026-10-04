use std::fmt;
use std::time::Instant;

use super::{commands::Commands, Command, CommandQueue, CommandQueueMetrics, DeferredSystemKey};

/// A per-system deferred-command buffer produced outside the World write window.
///
/// The compiled schedule owns the dispatch key. Worker completion order is
/// intentionally irrelevant: `CommandQueue::merge_worker_buffers` sorts by it
/// before moving commands into the World-owned queue.
#[derive(Debug)]
pub struct WorkerCommandBuffer {
    key: DeferredSystemKey,
    queue: CommandQueue,
    arena_is_in_destination: bool,
    spawn_generation: u64,
    next_spawn_ordinal: u32,
    next_run_generation: u64,
}

impl WorkerCommandBuffer {
    pub fn with_capacity(
        system_order: i32,
        system_id: impl Into<String>,
        command_capacity: usize,
    ) -> Self {
        let system_id = system_id.into();
        Self {
            key: DeferredSystemKey::registration_placeholder(system_order, system_id),
            queue: CommandQueue::with_capacity(command_capacity),
            arena_is_in_destination: false,
            spawn_generation: 0,
            next_spawn_ordinal: 0,
            next_run_generation: 0,
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

    pub fn metrics(&self) -> CommandQueueMetrics {
        self.queue.metrics()
    }

    /// Releases retained inline block backing storage when this producer has
    /// no queued commands. Schedule maintenance owns when to request it.
    pub fn trim_retained_inline_storage(&mut self) -> usize {
        self.queue.trim_retained_inline_storage()
    }

    /// The schedule runner injects the topologically compiled key before the
    /// callback can enqueue work. Registration metadata is only a prewarm-time
    /// placeholder and must never decide production merge order.
    pub(crate) fn bind_compiled_key(&mut self, key: DeferredSystemKey) {
        assert!(
            self.queue.is_empty(),
            "worker command buffer key cannot change while commands are queued"
        );
        self.key = key;
    }

    /// Starts a new producer window without changing its compiled schedule
    /// identity. Main-thread typed systems use this path, while worker systems
    /// set their compiled key immediately before calling it.
    pub(crate) fn begin_run(&mut self) {
        assert!(
            self.queue.is_empty(),
            "worker command buffer cannot begin a new run while commands are queued"
        );
        self.spawn_generation = self.next_run_generation;
        self.next_run_generation = self
            .next_run_generation
            .checked_add(1)
            .expect("worker deferred command run generation exhausted");
        self.next_spawn_ordinal = 0;
    }

    pub(crate) fn commands(&mut self) -> Commands<'_> {
        let key = self.key.clone();
        Commands::new(
            &mut self.queue,
            key,
            self.spawn_generation,
            &mut self.next_spawn_ordinal,
        )
    }

    pub(crate) fn merge_into(&mut self, destination: &mut CommandQueue) {
        if self.queue.is_empty() {
            destination.merge_empty_worker_metrics(&mut self.queue);
            return;
        }
        destination.append_worker(&mut self.queue, &self.key);
        self.arena_is_in_destination = true;
    }

    fn merge_into_with_known_absent_arena(&mut self, destination: &mut CommandQueue) {
        if self.queue.is_empty() {
            destination.merge_empty_worker_metrics(&mut self.queue);
            return;
        }
        destination.append_worker_with_known_absent_arena(&mut self.queue, &self.key);
        self.arena_is_in_destination = true;
    }

    pub(crate) fn reclaim_after_apply(&mut self, destination: &mut CommandQueue) {
        if !self.arena_is_in_destination {
            return;
        }
        if !matches!(
            destination.reclaim_worker_arena(&mut self.queue, &self.key),
            WorkerArenaReclaim::Pending
        ) {
            self.arena_is_in_destination = false;
        }
    }

    pub(crate) fn discard_pending(&mut self) {
        self.queue.discard_pending();
    }

    fn key(&self) -> &DeferredSystemKey {
        &self.key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCommandBufferMergeError {
    key: DeferredSystemKey,
}

impl WorkerCommandBufferMergeError {
    fn duplicate(buffer: &WorkerCommandBuffer) -> Self {
        Self {
            key: buffer.key.clone(),
        }
    }

    pub fn plan_order(&self) -> i32 {
        self.key.plan_order()
    }

    pub fn system_id(&self) -> &str {
        self.key.system_id()
    }
}

impl fmt::Display for WorkerCommandBufferMergeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate worker command buffer key ({}, {})",
            self.key.plan_order(),
            self.key.system_id()
        )
    }
}

impl std::error::Error for WorkerCommandBufferMergeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorkerArenaReclaim {
    Absent,
    Pending,
    Reclaimed,
}

impl CommandQueue {
    /// Moves worker-local command buffers into this World-owned queue in compiled
    /// schedule order. Duplicate schedule keys are rejected before any payload moves.
    pub fn merge_worker_buffers(
        &mut self,
        buffers: &mut [WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        if buffers.is_empty() {
            return Ok(());
        }
        let started_at = Instant::now();
        buffers.sort_by(|left, right| left.key().cmp(&right.key()));
        for pair in buffers.windows(2) {
            if pair[0].key() == pair[1].key() {
                return Err(WorkerCommandBufferMergeError::duplicate(&pair[0]));
            }
        }
        let destination_has_worker_arenas = self.has_worker_inline_arenas();
        for buffer in buffers {
            if destination_has_worker_arenas {
                buffer.merge_into(self);
            } else {
                buffer.merge_into_with_known_absent_arena(self);
            }
        }
        self.record_worker_batch_merge(started_at.elapsed());
        Ok(())
    }

    pub(crate) fn merge_worker_buffer_refs(
        &mut self,
        buffers: &mut [&mut WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        if buffers.is_empty() {
            return Ok(());
        }
        let started_at = Instant::now();
        buffers.sort_by(|left, right| left.key().cmp(&right.key()));
        for pair in buffers.windows(2) {
            if pair[0].key() == pair[1].key() {
                return Err(WorkerCommandBufferMergeError::duplicate(pair[0]));
            }
        }
        let destination_has_worker_arenas = self.has_worker_inline_arenas();
        for buffer in buffers {
            if destination_has_worker_arenas {
                buffer.merge_into(self);
            } else {
                buffer.merge_into_with_known_absent_arena(self);
            }
        }
        self.record_worker_batch_merge(started_at.elapsed());
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/worker_command_buffer.rs"]
mod tests;
