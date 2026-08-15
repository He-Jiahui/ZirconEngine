use crate::scene::ecs::{
    ChangeTickWindow, SystemParamAccess, SystemParamError, WorkerCommandBuffer,
};
use crate::scene::World;

pub(crate) mod worldless_private {
    pub trait Sealed {}
}

pub trait SystemParam {
    type State;
    type Item<'world>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError>;

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world>;

    /// Returns the single deferred-command lane owned by this parameter
    /// composition, when it contains `CommandsParam`.
    fn deferred_command_buffer_mut(_state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
        None
    }

    fn record_performance_diagnostics(_world: &mut World, _state: &mut Self::State) {}
}

/// Restricts worker execution to parameters that can produce an item without
/// borrowing World. This marker is deliberately separate from `SystemParam`:
/// normal systems retain the complete parameter surface.
pub trait WorldlessSystemParam: SystemParam + worldless_private::Sealed {
    fn get_param_without_world<'world>(state: &'world mut Self::State) -> Self::Item<'world>;
}

impl SystemParam for () {
    type State = ();
    type Item<'world> = ();

    fn init_state(
        _world: &mut World,
        _access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    unsafe fn get_param<'world>(
        _world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
    }
}

impl worldless_private::Sealed for () {}

impl WorldlessSystemParam for () {
    fn get_param_without_world<'world>(_state: &'world mut Self::State) -> Self::Item<'world> {}
}

macro_rules! tuple_system_param {
    ($($name:ident),*) => {
        impl<$($name),*> SystemParam for ($($name,)*)
        where
            $($name: SystemParam,)*
        {
            type State = ($($name::State,)*);
            type Item<'world> = ($($name::Item<'world>,)*);

            fn init_state(
                world: &mut World,
                access: &mut SystemParamAccess,
            ) -> Result<Self::State, SystemParamError> {
                Ok(($($name::init_state(world, access)?,)*))
            }

            #[allow(non_snake_case)]
            unsafe fn get_param<'world>(
                world: *mut World,
                state: &'world mut Self::State,
                ticks: ChangeTickWindow,
            ) -> Self::Item<'world> {
                let ($($name,)*) = state;
                ($($name::get_param(world, $name, ticks),)*)
            }

            #[allow(non_snake_case)]
            fn record_performance_diagnostics(world: &mut World, state: &mut Self::State) {
                let ($($name,)*) = state;
                $($name::record_performance_diagnostics(world, $name);)*
            }

            #[allow(non_snake_case)]
            fn deferred_command_buffer_mut(state: &mut Self::State) -> Option<&mut WorkerCommandBuffer> {
                let ($($name,)*) = state;
                let mut command_buffer = None;
                $(
                    if command_buffer.is_none() {
                        command_buffer = $name::deferred_command_buffer_mut($name);
                    }
                )*
                command_buffer
            }
        }

        impl<$($name),*> worldless_private::Sealed for ($($name,)*)
        where
            $($name: WorldlessSystemParam,)*
        {}

        impl<$($name),*> WorldlessSystemParam for ($($name,)*)
        where
            $($name: WorldlessSystemParam,)*
        {
            #[allow(non_snake_case)]
            fn get_param_without_world<'world>(state: &'world mut Self::State) -> Self::Item<'world> {
                let ($($name,)*) = state;
                ($($name::get_param_without_world($name),)*)
            }

        }
    };
}

tuple_system_param!(A);
tuple_system_param!(A, B);
tuple_system_param!(A, B, C);
tuple_system_param!(A, B, C, D);
tuple_system_param!(A, B, C, D, E);
tuple_system_param!(A, B, C, D, E, F);
tuple_system_param!(A, B, C, D, E, F, G);
tuple_system_param!(A, B, C, D, E, F, G, H);
