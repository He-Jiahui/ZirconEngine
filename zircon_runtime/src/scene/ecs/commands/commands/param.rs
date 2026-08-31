use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, WorkerCommandBuffer,
    WorldlessSystemParam,
};

use super::facade::Commands;

pub struct CommandsParam;

#[derive(Debug)]
pub struct CommandsParamState {
    worker_commands: WorkerCommandBuffer,
}

impl Default for CommandsParam {
    fn default() -> Self {
        Self
    }
}

impl SystemParam for CommandsParam {
    type State = CommandsParamState;
    type Item<'world> = Commands<'world>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_deferred_commands()?;
        let key = world.allocate_direct_system_deferred_key();
        Ok(CommandsParamState {
            worker_commands: WorkerCommandBuffer::with_capacity(
                key.plan_order(),
                key.system_id(),
                0,
            ),
        })
    }

    unsafe fn get_param<'world>(
        _world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        state.worker_commands.commands()
    }

    fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        Some(&mut state.worker_commands)
    }
}

impl crate::scene::ecs::worldless_private::Sealed for CommandsParam {}

impl WorldlessSystemParam for CommandsParam {
    fn get_param_without_world<'world>(state: &'world mut Self::State) -> Self::Item<'world> {
        state.worker_commands.commands()
    }
}
