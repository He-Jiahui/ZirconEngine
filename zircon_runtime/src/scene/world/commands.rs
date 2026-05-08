use crate::scene::ecs::{CommandQueue, Commands};
use crate::scene::World;

impl World {
    pub fn commands(&mut self) -> Commands<'_> {
        Commands::new(&mut self.command_queue)
    }

    pub fn apply_deferred(&mut self) {
        if self.command_queue.is_empty() {
            return;
        }
        let mut queue = std::mem::take(&mut self.command_queue);
        let tick = self.advance_change_tick();
        let previous_active_tick = self.replace_active_change_tick(Some(tick));
        queue.apply(self);
        self.replace_active_change_tick(previous_active_tick);
    }

    pub fn has_deferred_commands(&self) -> bool {
        !self.command_queue.is_empty()
    }

    pub(crate) fn command_queue_mut(&mut self) -> &mut CommandQueue {
        &mut self.command_queue
    }
}
