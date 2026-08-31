use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::mem::size_of;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::time::{Duration, Instant};

use crate::scene::World;

use super::inline_command_arena::{InlineCommandArena, WorkerInlineCommandArena};
use super::queued_command::{InlineCommand, QueuedCommand};
use super::worker_command_buffer::WorkerArenaReclaim;
use super::{
    Command, CommandQueueMetrics, DeferredCommandReport, DeferredSystemKey, QueuedStructuralCommand,
};

/// The World-bound deferred-command owner. Payload storage and queued-entry
/// mechanics live in sibling leaf modules so this type only owns barriers.
#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<QueuedCommand>,
    inline_arena: InlineCommandArena,
    // Worker arenas stay separate until the barrier has drained them, so their
    // physical blocks can return to the producer that prewarmed them.
    worker_inline_arenas: Vec<WorkerInlineCommandArena>,
    metrics: CommandQueueMetrics,
}

impl CommandQueue {
    pub fn with_capacity(command_capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(command_capacity),
            inline_arena: InlineCommandArena::with_command_capacity(command_capacity),
            worker_inline_arenas: Vec::new(),
            metrics: CommandQueueMetrics::with_command_capacity(command_capacity),
        }
    }

    pub fn push<C>(&mut self, command: C)
    where
        C: Command,
    {
        self.push_with(
            command,
            InlineCommandArena::try_push::<C>,
            |command, bytes| QueuedCommand::Fallback(Box::new(command), bytes),
        );
    }

    pub(crate) fn push_structural<C>(&mut self, command: C)
    where
        C: QueuedStructuralCommand,
    {
        self.push_with(
            command,
            InlineCommandArena::try_push_structural::<C>,
            |command, bytes| QueuedCommand::StructuralFallback(Box::new(command), bytes),
        );
    }

    fn push_with<C>(
        &mut self,
        command: C,
        inline_push: fn(&mut InlineCommandArena, C) -> Result<(InlineCommand, usize, bool), C>,
        fallback: impl FnOnce(C, usize) -> QueuedCommand,
    ) where
        C: Command,
    {
        let payload_bytes = size_of::<C>();
        let command = match inline_push(&mut self.inline_arena, command) {
            Ok((command, storage_bytes, storage_grew)) => {
                if storage_grew {
                    self.metrics.inline_block_storage_grew();
                }
                QueuedCommand::Inline {
                    command,
                    payload_bytes,
                    storage_bytes,
                }
            }
            Err(command) => fallback(command, payload_bytes),
        };
        self.metrics
            .queued(command.storage().expect("new command must own storage"));
        if self.commands.len() == self.commands.capacity() {
            self.metrics.queue_storage_grew();
        }
        self.commands.push(command);
    }

    pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport {
        let started_at = Instant::now();
        let applied_count = self.commands.len();
        world.clear_deferred_command_errors();

        let mut spawn_tokens = BTreeSet::new();
        for command in &self.commands {
            // Only a structural spawn produces a token. A later command that
            // merely references an old public handle cannot resurrect it or
            // claim another entity id in this window.
            command.collect_spawn_tokens(
                &self.inline_arena,
                &self.worker_inline_arenas,
                &mut spawn_tokens,
            );
        }
        let resolved_entities = match world.reserve_deferred_spawn_tokens(spawn_tokens) {
            Ok(resolved_entities) => resolved_entities,
            Err(error) => {
                // This window cannot be retried with a different partial id
                // plan. Consume it exactly once, retaining the next window
                // that World::apply_deferred has already swapped aside.
                self.discard_pending();
                self.metrics.record_world_apply(started_at.elapsed());
                return DeferredCommandReport::new(
                    applied_count,
                    Vec::new(),
                    Some(error),
                    BTreeMap::new(),
                );
            }
        };
        world.install_deferred_spawn_resolutions(&resolved_entities);

        let (commands, inline_arena, worker_inline_arenas, metrics) = (
            &mut self.commands,
            &mut self.inline_arena,
            &mut self.worker_inline_arenas,
            &mut self.metrics,
        );
        let mut cursor = 0;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut structural_batch = None;
            while cursor < commands.len() {
                let command = std::mem::replace(&mut commands[cursor], QueuedCommand::Consumed);
                cursor += 1;
                let storage = command
                    .storage()
                    .expect("unconsumed queue entry must own storage");
                metrics.dispatched(storage);

                if command
                    .structural_metadata(inline_arena, worker_inline_arenas)
                    .is_some()
                {
                    let batch = structural_batch
                        .get_or_insert_with(crate::scene::world::DeferredStructuralBatch::new);
                    command.stage_structural(inline_arena, worker_inline_arenas, batch, world);
                    continue;
                }

                if let Some(batch) = structural_batch.take() {
                    for error in batch.finish(world) {
                        world.record_deferred_command_error(error);
                    }
                }
                command.apply(inline_arena, worker_inline_arenas, world);
            }

            if let Some(batch) = structural_batch.take() {
                for error in batch.finish(world) {
                    world.record_deferred_command_error(error);
                }
            }
        }));

        if let Err(payload) = result {
            for command in commands.iter_mut().skip(cursor) {
                let command = std::mem::replace(command, QueuedCommand::Consumed);
                if let Some(storage) = command.storage() {
                    metrics.discarded(storage);
                    command.discard(inline_arena, worker_inline_arenas);
                }
            }
            commands.clear();
            inline_arena.reset();
            for worker_arena in worker_inline_arenas {
                worker_arena.arena.reset();
            }
            world.clear_deferred_spawn_resolutions();
            metrics.record_world_apply(started_at.elapsed());
            resume_unwind(payload);
        }

        commands.clear();
        inline_arena.reset();
        for worker_arena in worker_inline_arenas {
            worker_arena.arena.reset();
        }
        let published_entities = world.take_published_deferred_spawn_resolutions();
        world.clear_deferred_spawn_resolutions();
        metrics.record_world_apply(started_at.elapsed());
        DeferredCommandReport::new(
            applied_count,
            world.take_deferred_command_errors(),
            None,
            published_entities,
        )
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub(super) fn has_worker_inline_arenas(&self) -> bool {
        !self.worker_inline_arenas.is_empty()
    }

    pub fn metrics(&self) -> CommandQueueMetrics {
        self.metrics
    }

    /// Explicitly returns retained inline block backing storage to the
    /// allocator when this queue has no live commands. This is intentionally
    /// separate from `apply` so ordinary frame-to-frame reuse remains intact.
    pub fn trim_retained_inline_storage(&mut self) -> usize {
        if !self.is_empty() {
            return 0;
        }
        let mut released_bytes = self.inline_arena.trim_idle_storage();
        for worker_arena in &mut self.worker_inline_arenas {
            released_bytes = released_bytes.saturating_add(worker_arena.arena.trim_idle_storage());
        }
        self.metrics.inline_arena_storage_trimmed(released_bytes);
        released_bytes
    }

    pub(super) fn record_worker_batch_merge(&mut self, elapsed: Duration) {
        self.metrics.record_worker_batch_merge(elapsed);
    }

    pub(super) fn merge_empty_worker_metrics(&mut self, other: &mut Self) {
        debug_assert!(other.commands.is_empty());
        debug_assert!(other.worker_inline_arenas.is_empty());
        self.metrics.merge_from(other.metrics);
        other.metrics = CommandQueueMetrics::default();
    }

    pub(crate) fn discard_pending(&mut self) {
        for command in &mut self.commands {
            let command = std::mem::replace(command, QueuedCommand::Consumed);
            if let Some(storage) = command.storage() {
                self.metrics.discarded(storage);
                command.discard(&mut self.inline_arena, &mut self.worker_inline_arenas);
            }
        }
        self.commands.clear();
        self.inline_arena.reset();
        for worker_arena in &mut self.worker_inline_arenas {
            worker_arena.arena.reset();
        }
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        let required_capacity = self.commands.len().saturating_add(other.commands.len());
        if required_capacity > self.commands.capacity() {
            self.metrics.queue_storage_grew();
        }
        let (queue_block_offset, storage_grew, queue_leading_padding) =
            self.inline_arena.append(&mut other.inline_arena);
        if storage_grew {
            self.metrics.inline_block_storage_grew();
        }
        if queue_leading_padding != 0 {
            let mut prefix_recorded = false;
            for command in &mut other.commands {
                if command.add_queue_inline_storage_prefix(queue_leading_padding) {
                    prefix_recorded = true;
                    break;
                }
            }
            debug_assert!(prefix_recorded);
            other
                .metrics
                .add_queued_inline_storage_padding(queue_leading_padding);
        }

        let source_worker_arenas = std::mem::take(&mut other.worker_inline_arenas);
        let mut worker_arena_remaps = Vec::with_capacity(source_worker_arenas.len());
        for mut source_worker_arena in source_worker_arenas {
            let source_index = worker_arena_remaps.len();
            let matching_destination = self
                .worker_inline_arenas
                .iter()
                .position(|arena| arena.matches(&source_worker_arena.key));
            if let Some(destination_index) = matching_destination {
                let (block_offset, storage_grew, leading_padding) = self.worker_inline_arenas
                    [destination_index]
                    .arena
                    .append(&mut source_worker_arena.arena);
                if storage_grew {
                    self.metrics.inline_block_storage_grew();
                }
                if leading_padding != 0 {
                    let mut prefix_recorded = false;
                    for command in &mut other.commands {
                        if command.add_worker_inline_storage_prefix(source_index, leading_padding) {
                            prefix_recorded = true;
                            break;
                        }
                    }
                    debug_assert!(prefix_recorded);
                    other
                        .metrics
                        .add_queued_inline_storage_padding(leading_padding);
                }
                worker_arena_remaps.push((destination_index, block_offset));
            } else {
                let destination_index = self.worker_inline_arenas.len();
                self.worker_inline_arenas.push(source_worker_arena);
                worker_arena_remaps.push((destination_index, 0));
            }
        }

        if queue_block_offset != 0 || !worker_arena_remaps.is_empty() {
            for command in &mut other.commands {
                command.remap_appended_arena(queue_block_offset, &worker_arena_remaps);
            }
        }
        self.commands.append(&mut other.commands);
        self.metrics.merge_from(other.metrics);
        other.metrics = CommandQueueMetrics::default();
    }

    pub(crate) fn append_worker(&mut self, other: &mut Self, key: &DeferredSystemKey) {
        debug_assert!(other.worker_inline_arenas.is_empty());
        let required_capacity = self.commands.len().saturating_add(other.commands.len());
        if required_capacity > self.commands.capacity() {
            self.metrics.queue_storage_grew();
        }

        let worker_arena_index = self
            .worker_inline_arenas
            .iter()
            .position(|arena| arena.matches(key));
        let (worker_arena_index, block_offset, storage_grew, leading_padding) =
            if let Some(index) = worker_arena_index {
                let arena = &mut self.worker_inline_arenas[index].arena;
                let (block_offset, storage_grew, leading_padding) =
                    arena.append(&mut other.inline_arena);
                (index, block_offset, storage_grew, leading_padding)
            } else {
                let index = self.worker_inline_arenas.len();
                self.worker_inline_arenas.push(WorkerInlineCommandArena {
                    key: key.clone(),
                    arena: std::mem::take(&mut other.inline_arena),
                });
                (index, 0, false, 0)
            };

        self.finish_appending_worker(
            other,
            worker_arena_index,
            block_offset,
            storage_grew,
            leading_padding,
        );
    }

    /// Appends one known-distinct worker arena without searching the destination.
    /// `merge_worker_buffers` sorts and rejects duplicate keys before using this
    /// fast path for an otherwise arena-free command queue.
    pub(super) fn append_worker_with_known_absent_arena(
        &mut self,
        other: &mut Self,
        key: &DeferredSystemKey,
    ) {
        debug_assert!(other.worker_inline_arenas.is_empty());
        let worker_arena_index = self.worker_inline_arenas.len();
        self.worker_inline_arenas.push(WorkerInlineCommandArena {
            key: key.clone(),
            arena: std::mem::take(&mut other.inline_arena),
        });
        self.finish_appending_worker(other, worker_arena_index, 0, false, 0);
    }

    fn finish_appending_worker(
        &mut self,
        other: &mut Self,
        worker_arena_index: usize,
        block_offset: usize,
        storage_grew: bool,
        leading_padding: usize,
    ) {
        if storage_grew {
            self.metrics.inline_block_storage_grew();
        }
        if leading_padding != 0 {
            let mut prefix_recorded = false;
            for command in &mut other.commands {
                if command.add_queue_inline_storage_prefix(leading_padding) {
                    prefix_recorded = true;
                    break;
                }
            }
            debug_assert!(prefix_recorded);
            other
                .metrics
                .add_queued_inline_storage_padding(leading_padding);
        }
        for command in &mut other.commands {
            command.remap_inline_to_worker(worker_arena_index, block_offset);
        }
        self.commands.append(&mut other.commands);
        self.metrics.merge_from(other.metrics);
        other.metrics = CommandQueueMetrics::default();
    }

    pub(crate) fn reclaim_worker_arena(
        &mut self,
        worker_queue: &mut Self,
        key: &DeferredSystemKey,
    ) -> WorkerArenaReclaim {
        let Some(index) = self
            .worker_inline_arenas
            .iter()
            .position(|arena| arena.matches(key))
        else {
            return WorkerArenaReclaim::Absent;
        };
        if self
            .commands
            .iter()
            .any(|command| command.references_worker_arena(index))
        {
            return WorkerArenaReclaim::Pending;
        }
        assert!(
            worker_queue.is_empty(),
            "worker command buffer must be empty before its arena returns"
        );
        let last_index = self.worker_inline_arenas.len() - 1;
        let worker_arena = self.worker_inline_arenas.swap_remove(index);
        if index != last_index {
            for command in &mut self.commands {
                command.remap_worker_arena(last_index, index);
            }
        }
        worker_queue.inline_arena = worker_arena.arena;
        WorkerArenaReclaim::Reclaimed
    }
}

impl fmt::Debug for CommandQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandQueue")
            .field("len", &self.commands.len())
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl Drop for CommandQueue {
    fn drop(&mut self) {
        self.discard_pending();
    }
}

impl Clone for CommandQueue {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for CommandQueue {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandQueue, World};

    #[test]
    fn ecs_commands_apply_records_one_world_owned_boundary() {
        let mut queue = CommandQueue::default();
        queue.push(|_: &mut World| {});

        queue.apply(&mut World::empty());

        assert_eq!(queue.metrics().world_apply_count(), 1);
    }
}
