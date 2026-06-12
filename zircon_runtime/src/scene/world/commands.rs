use crate::scene::ecs::{CommandQueue, Commands, DeferredCommandError, DeferredCommandReport};
use crate::scene::World;

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
        let report = queue.apply(self);
        self.replace_active_change_tick(previous_active_tick);
        report
    }

    pub fn has_deferred_commands(&self) -> bool {
        !self.command_queue.is_empty()
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
