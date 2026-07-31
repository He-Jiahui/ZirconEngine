use crate::scene::World;
use crate::scene::ecs::{ChangeTickWindow, SystemParamAccess, SystemParamError};

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

    fn record_performance_diagnostics(_world: &mut World, _state: &mut Self::State) {}
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
