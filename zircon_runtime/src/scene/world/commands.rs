use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use crate::scene::World;
use crate::scene::ecs::{
    CommandQueue, Commands, DeferredCommandError, DeferredCommandReport, WorkerCommandBuffer,
    WorkerCommandBufferMergeError,
};

impl World {
    pub fn commands(&mut self) -> Commands<'_> {
        let (queue, next_entity) = self.command_state_mut();
        Commands::new(queue, next_entity)
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

    pub(crate) fn merge_worker_command_buffer(&mut self, buffer: &mut WorkerCommandBuffer) {
        buffer.merge_into(&mut self.command_queue);
    }

    pub(crate) fn merge_worker_command_buffers(
        &mut self,
        buffers: &mut [&mut WorkerCommandBuffer],
    ) -> Result<(), WorkerCommandBufferMergeError> {
        self.command_queue.merge_worker_buffer_refs(buffers)
    }

    pub(crate) fn command_state_mut(&mut self) -> (&mut CommandQueue, &mut crate::scene::EntityId) {
        (&mut self.command_queue, &mut self.next_id)
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
}
