use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use crate::scene::World;
use crate::scene::ecs::{
    CommandQueue, CommandQueueMetrics, Commands, DeferredCommandError, DeferredCommandReport,
    DeferredCommandTarget, DeferredEntityRef, DeferredSpawnToken, DeferredSystemKey,
    WorkerCommandBuffer, WorkerCommandBufferMergeError,
};

impl World {
    pub(crate) fn allocate_direct_system_deferred_key(&mut self) -> DeferredSystemKey {
        let ordinal = self.deferred_direct_system_ordinal;
        self.deferred_direct_system_ordinal = self
            .deferred_direct_system_ordinal
            .checked_add(1)
            .expect("direct deferred system lane ordinal exhausted");
        DeferredSystemKey::direct_system(ordinal)
    }

    pub fn commands(&mut self) -> Commands<'_> {
        Commands::new(
            &mut self.command_queue,
            DeferredSystemKey::direct_world(),
            0,
            &mut self.deferred_direct_spawn_ordinal,
        )
    }

    pub fn apply_deferred(&mut self) -> DeferredCommandReport {
        if self.command_queue.is_empty() {
            return DeferredCommandReport::default();
        }
        let mut queue = std::mem::take(&mut self.command_queue);
        let tick = self.advance_change_tick();
        let previous_active_tick = self.replace_active_change_tick(Some(tick));
        let result = catch_unwind(AssertUnwindSafe(|| queue.apply(self)));
        self.replace_active_change_tick(previous_active_tick);
        queue.append(&mut self.command_queue);
        self.command_queue = queue;

        match result {
            Ok(report) => report,
            Err(payload) => resume_unwind(payload),
        }
    }

    pub fn has_deferred_commands(&self) -> bool {
        !self.command_queue.is_empty()
    }

    /// Returns current counters for the World-owned deferred command queue.
    ///
    /// Wall-clock boundary timing is available here. Operating-system lock
    /// counts remain profiler-owned evidence from the managed Windows ETW pass.
    pub fn deferred_command_metrics(&self) -> CommandQueueMetrics {
        self.command_queue.metrics()
    }

    /// Releases idle deferred-command inline backing storage on an explicit
    /// maintenance or pressure path without changing normal frame reuse.
    pub fn trim_deferred_command_storage(&mut self) -> usize {
        self.command_queue.trim_retained_inline_storage()
    }

    pub(crate) fn merge_worker_command_buffer(&mut self, buffer: &mut WorkerCommandBuffer) {
        buffer.merge_into(&mut self.command_queue);
    }

    pub(crate) fn reclaim_worker_command_buffer(&mut self, buffer: &mut WorkerCommandBuffer) {
        buffer.reclaim_after_apply(&mut self.command_queue);
    }

    pub(crate) fn merge_worker_command_buffers(
        &mut self,
        buffers: &mut [&mut WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        self.command_queue.merge_worker_buffer_refs(buffers)
    }

    pub(crate) fn reclaim_worker_command_buffers(
        &mut self,
        buffers: &mut [&mut WorkerCommandBuffer],
    ) {
        for buffer in buffers {
            self.reclaim_worker_command_buffer(buffer);
        }
    }

    pub(crate) fn record_deferred_command_error(&mut self, error: DeferredCommandError) {
        self.deferred_command_errors.push(error);
    }

    pub(crate) fn clear_deferred_command_errors(&mut self) {
        self.deferred_command_errors.clear();
    }

    pub(crate) fn take_deferred_command_errors(&mut self) -> Vec<DeferredCommandError> {
        std::mem::take(&mut self.deferred_command_errors)
    }

    pub(crate) fn reserve_deferred_spawn_tokens(
        &mut self,
        tokens: BTreeSet<DeferredSpawnToken>,
    ) -> Result<BTreeMap<DeferredSpawnToken, crate::scene::EntityId>, crate::scene::SceneError>
    {
        let mut allocator = self.entity_id_allocator;
        let mut resolved = BTreeMap::new();
        for token in tokens {
            let entity = allocator.reserve_next()?;
            resolved.insert(token, entity);
        }
        self.entity_id_allocator = allocator;
        Ok(resolved)
    }

    pub(crate) fn install_deferred_spawn_resolutions(
        &mut self,
        resolutions: &BTreeMap<DeferredSpawnToken, crate::scene::EntityId>,
    ) {
        self.deferred_spawn_resolutions.clone_from(resolutions);
        self.published_deferred_spawns.clear();
    }

    pub(crate) fn clear_deferred_spawn_resolutions(&mut self) {
        self.deferred_spawn_resolutions.clear();
        self.published_deferred_spawns.clear();
    }

    pub(crate) fn resolve_deferred_entity_ref(
        &self,
        target: &DeferredEntityRef,
    ) -> Option<crate::scene::EntityId> {
        match target {
            DeferredEntityRef::Existing(entity) => Some(*entity),
            DeferredEntityRef::Spawn(token) => self.deferred_spawn_resolutions.get(token).copied(),
        }
    }

    pub(crate) fn deferred_command_target(
        &self,
        target: &DeferredEntityRef,
    ) -> DeferredCommandTarget {
        match target {
            DeferredEntityRef::Existing(entity) => DeferredCommandTarget::resolved(*entity),
            DeferredEntityRef::Spawn(token) => self
                .deferred_spawn_resolutions
                .get(token)
                .copied()
                .map(DeferredCommandTarget::resolved)
                .unwrap_or_else(|| DeferredCommandTarget::pending(token.clone())),
        }
    }

    pub(crate) fn mark_deferred_spawn_published(&mut self, token: DeferredSpawnToken) {
        self.published_deferred_spawns.insert(token);
    }

    pub(crate) fn take_published_deferred_spawn_resolutions(
        &self,
    ) -> BTreeMap<DeferredSpawnToken, crate::scene::EntityId> {
        self.published_deferred_spawns
            .iter()
            .filter_map(|token| {
                self.deferred_spawn_resolutions
                    .get(token)
                    .copied()
                    .map(|entity| (token.clone(), entity))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::World;

    #[test]
    fn ecs_commands_deferred_metrics_observe_only_entered_apply_boundaries() {
        let mut world = World::empty();

        assert_eq!(world.apply_deferred().applied_count(), 0);
        assert_eq!(world.deferred_command_metrics().world_apply_count(), 0);

        world.commands().queue_fn(|_: &mut World| {});
        assert_eq!(world.apply_deferred().applied_count(), 1);
        assert_eq!(world.deferred_command_metrics().world_apply_count(), 1);
    }
}
