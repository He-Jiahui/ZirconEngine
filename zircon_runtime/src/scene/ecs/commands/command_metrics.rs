use std::time::Duration;

use super::queued_command::QueuedCommandStorage;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandQueueMetrics {
    queued_inline_commands: usize,
    queued_fallback_commands: usize,
    queued_inline_bytes: usize,
    queued_inline_storage_bytes: usize,
    queued_fallback_bytes: usize,
    queue_storage_growths: usize,
    inline_block_storage_growths: usize,
    inline_arena_trim_count: usize,
    inline_arena_trimmed_storage_bytes: usize,
    fallback_payload_allocations: usize,
    fallback_payload_releases: usize,
    inline_payload_releases: usize,
    inline_dispatch_calls: usize,
    fallback_dispatch_calls: usize,
    discarded_inline_commands: usize,
    discarded_fallback_commands: usize,
    worker_batch_merge_count: usize,
    worker_batch_merge_ns: u64,
    world_apply_count: usize,
    world_apply_ns: u64,
}

impl CommandQueueMetrics {
    pub(super) fn with_command_capacity(command_capacity: usize) -> Self {
        Self {
            queue_storage_growths: usize::from(command_capacity > 0),
            inline_block_storage_growths: usize::from(command_capacity > 0),
            ..Self::default()
        }
    }

    pub fn queued_inline_commands(&self) -> usize {
        self.queued_inline_commands
    }

    pub fn queued_fallback_commands(&self) -> usize {
        self.queued_fallback_commands
    }

    pub fn queued_inline_bytes(&self) -> usize {
        self.queued_inline_bytes
    }

    /// Occupied packed-arena bytes, including alignment and block-tail padding.
    pub fn queued_inline_storage_bytes(&self) -> usize {
        self.queued_inline_storage_bytes
    }

    pub fn queued_fallback_bytes(&self) -> usize {
        self.queued_fallback_bytes
    }

    /// Counts backing-vector growth, not one allocation per inline payload.
    pub fn queue_storage_growths(&self) -> usize {
        self.queue_storage_growths
    }

    /// Counts packed block-vector allocations or growth, never individual commands.
    pub fn inline_block_storage_growths(&self) -> usize {
        self.inline_block_storage_growths
    }

    /// Number of explicit idle/pressure trims that returned packed block storage.
    pub fn inline_arena_trim_count(&self) -> usize {
        self.inline_arena_trim_count
    }

    /// Logical packed-block backing bytes released by explicit trims.
    ///
    /// This is allocator-facing capacity, not a process RSS measurement.
    pub fn inline_arena_trimmed_storage_bytes(&self) -> usize {
        self.inline_arena_trimmed_storage_bytes
    }

    /// Counts explicit heap-backed fallback allocations only.
    pub fn fallback_payload_allocations(&self) -> usize {
        self.fallback_payload_allocations
    }

    /// Counts heap-backed fallback storage releases; zero-sized fallbacks allocate nothing.
    pub fn fallback_payload_releases(&self) -> usize {
        self.fallback_payload_releases
    }

    /// Counts consumed or panic-discarded inline payloads.
    pub fn inline_payload_releases(&self) -> usize {
        self.inline_payload_releases
    }

    pub fn inline_dispatch_calls(&self) -> usize {
        self.inline_dispatch_calls
    }

    pub fn fallback_dispatch_calls(&self) -> usize {
        self.fallback_dispatch_calls
    }

    pub fn discarded_inline_commands(&self) -> usize {
        self.discarded_inline_commands
    }

    pub fn discarded_fallback_commands(&self) -> usize {
        self.discarded_fallback_commands
    }

    /// Counts deterministic worker-lane merge barriers, not individual commands.
    pub fn worker_batch_merge_count(&self) -> usize {
        self.worker_batch_merge_count
    }

    /// Cumulative wall-clock time spent in successful worker merge barriers.
    ///
    /// This includes deterministic lane sorting, duplicate-key preflight, and
    /// ownership transfer into the World-owned queue.
    pub fn worker_batch_merge_duration(&self) -> Duration {
        Duration::from_nanos(self.worker_batch_merge_ns)
    }

    /// Counts deferred apply boundaries that enter the World-owned command queue.
    ///
    /// This is intentionally distinct from operating-system lock acquisitions,
    /// which the managed Windows ETW profiling pass observes around this boundary.
    pub fn world_apply_count(&self) -> usize {
        self.world_apply_count
    }

    /// Cumulative wall-clock time spent inside World-owned deferred application.
    pub fn world_apply_duration(&self) -> Duration {
        Duration::from_nanos(self.world_apply_ns)
    }

    pub(super) fn queued(&mut self, storage: QueuedCommandStorage) {
        match storage {
            QueuedCommandStorage::Inline {
                payload_bytes,
                storage_bytes,
            } => {
                self.queued_inline_commands += 1;
                self.queued_inline_bytes += payload_bytes;
                self.queued_inline_storage_bytes += storage_bytes;
            }
            QueuedCommandStorage::Fallback(bytes) => {
                self.queued_fallback_commands += 1;
                self.queued_fallback_bytes += bytes;
                self.fallback_payload_allocations += usize::from(bytes > 0);
            }
        }
    }

    pub(super) fn queue_storage_grew(&mut self) {
        self.queue_storage_growths += 1;
    }

    pub(super) fn inline_block_storage_grew(&mut self) {
        self.inline_block_storage_growths += 1;
    }

    pub(super) fn inline_arena_storage_trimmed(&mut self, released_bytes: usize) {
        if released_bytes == 0 {
            return;
        }
        self.inline_arena_trim_count += 1;
        self.inline_arena_trimmed_storage_bytes = self
            .inline_arena_trimmed_storage_bytes
            .saturating_add(released_bytes);
    }

    pub(super) fn add_queued_inline_storage_padding(&mut self, bytes: usize) {
        self.queued_inline_storage_bytes += bytes;
    }

    pub(super) fn dispatched(&mut self, storage: QueuedCommandStorage) {
        match storage {
            QueuedCommandStorage::Inline {
                payload_bytes,
                storage_bytes,
            } => {
                self.queued_inline_commands -= 1;
                self.queued_inline_bytes -= payload_bytes;
                self.queued_inline_storage_bytes -= storage_bytes;
                self.inline_payload_releases += 1;
                self.inline_dispatch_calls += 1;
            }
            QueuedCommandStorage::Fallback(bytes) => {
                self.queued_fallback_commands -= 1;
                self.queued_fallback_bytes -= bytes;
                self.fallback_payload_releases += usize::from(bytes > 0);
                self.fallback_dispatch_calls += 1;
            }
        }
    }

    pub(super) fn discarded(&mut self, storage: QueuedCommandStorage) {
        self.dispatched(storage);
        match storage {
            QueuedCommandStorage::Inline { .. } => {
                self.inline_dispatch_calls -= 1;
                self.discarded_inline_commands += 1;
            }
            QueuedCommandStorage::Fallback(_) => {
                self.fallback_dispatch_calls -= 1;
                self.discarded_fallback_commands += 1;
            }
        }
    }

    pub(super) fn record_worker_batch_merge(&mut self, elapsed: Duration) {
        self.worker_batch_merge_count += 1;
        self.worker_batch_merge_ns = self
            .worker_batch_merge_ns
            .saturating_add(duration_ns(elapsed));
    }

    pub(super) fn record_world_apply(&mut self, elapsed: Duration) {
        self.world_apply_count += 1;
        self.world_apply_ns = self.world_apply_ns.saturating_add(duration_ns(elapsed));
    }

    pub(super) fn merge_from(&mut self, other: Self) {
        self.queued_inline_commands += other.queued_inline_commands;
        self.queued_fallback_commands += other.queued_fallback_commands;
        self.queued_inline_bytes += other.queued_inline_bytes;
        self.queued_inline_storage_bytes += other.queued_inline_storage_bytes;
        self.queued_fallback_bytes += other.queued_fallback_bytes;
        self.queue_storage_growths += other.queue_storage_growths;
        self.inline_block_storage_growths += other.inline_block_storage_growths;
        self.inline_arena_trim_count += other.inline_arena_trim_count;
        self.inline_arena_trimmed_storage_bytes = self
            .inline_arena_trimmed_storage_bytes
            .saturating_add(other.inline_arena_trimmed_storage_bytes);
        self.fallback_payload_allocations += other.fallback_payload_allocations;
        self.fallback_payload_releases += other.fallback_payload_releases;
        self.inline_payload_releases += other.inline_payload_releases;
        self.inline_dispatch_calls += other.inline_dispatch_calls;
        self.fallback_dispatch_calls += other.fallback_dispatch_calls;
        self.discarded_inline_commands += other.discarded_inline_commands;
        self.discarded_fallback_commands += other.discarded_fallback_commands;
        self.worker_batch_merge_count += other.worker_batch_merge_count;
        self.worker_batch_merge_ns = self
            .worker_batch_merge_ns
            .saturating_add(other.worker_batch_merge_ns);
        self.world_apply_count += other.world_apply_count;
        self.world_apply_ns = self.world_apply_ns.saturating_add(other.world_apply_ns);
    }
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::CommandQueueMetrics;

    #[test]
    fn ecs_commands_queue_operation_timings_merge_with_transferred_metrics() {
        let mut source = CommandQueueMetrics::default();
        source.record_worker_batch_merge(Duration::from_nanos(7));
        source.record_world_apply(Duration::from_nanos(11));

        let mut destination = CommandQueueMetrics::default();
        destination.merge_from(source);

        assert_eq!(destination.worker_batch_merge_count(), 1);
        assert_eq!(
            destination.worker_batch_merge_duration(),
            Duration::from_nanos(7)
        );
        assert_eq!(destination.world_apply_count(), 1);
        assert_eq!(destination.world_apply_duration(), Duration::from_nanos(11));
    }
}
