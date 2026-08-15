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
        destination.append_worker(&mut self.queue, &self.key);
    }

    pub(crate) fn reclaim_after_apply(&mut self, destination: &mut CommandQueue) {
        destination.reclaim_worker_arena(&mut self.queue, &self.key);
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
        for buffer in buffers {
            buffer.merge_into(self);
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
        for buffer in buffers {
            buffer.merge_into(self);
        }
        self.record_worker_batch_merge(started_at.elapsed());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::World;

    use super::{CommandQueue, WorkerCommandBuffer};

    #[test]
    fn ecs_commands_successful_worker_barrier_records_one_merge() {
        let mut buffer = WorkerCommandBuffer::with_capacity(0, "commands.metrics", 0);
        buffer.push(|_: &mut World| {});
        let mut queue = CommandQueue::default();

        queue
            .merge_worker_buffers(std::slice::from_mut(&mut buffer))
            .expect("one unique worker key must merge");
        queue.apply(&mut World::empty());

        assert_eq!(queue.metrics().worker_batch_merge_count(), 1);
        assert_eq!(queue.metrics().world_apply_count(), 1);
    }

    #[test]
    fn ecs_commands_empty_worker_barrier_does_not_record_a_merge() {
        let mut queue = CommandQueue::default();

        queue
            .merge_worker_buffers(&mut [])
            .expect("an empty worker slice is a no-op");

        assert_eq!(queue.metrics().worker_batch_merge_count(), 0);
    }

    #[test]
    fn ecs_commands_rejected_worker_barrier_does_not_record_a_merge() {
        let mut buffers = [
            WorkerCommandBuffer::with_capacity(0, "commands.metrics.duplicate", 0),
            WorkerCommandBuffer::with_capacity(0, "commands.metrics.duplicate", 0),
        ];
        let mut queue = CommandQueue::default();

        assert!(queue.merge_worker_buffers(&mut buffers).is_err());

        assert_eq!(queue.metrics().worker_batch_merge_count(), 0);
    }
}
