use crate::scene::World;
use crate::scene::ecs::{ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError};

use super::facade::Commands;

pub struct CommandsParam;

impl Default for CommandsParam {
    fn default() -> Self {
        Self
    }
}

impl SystemParam for CommandsParam {
    type State = ();
    type Item<'world> = Commands<'world>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_deferred_commands();
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        let (queue, next_entity) = world.command_state_mut();
        Commands::new(queue, next_entity)
    }
}
