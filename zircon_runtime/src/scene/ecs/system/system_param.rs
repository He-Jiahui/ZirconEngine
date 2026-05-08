use crate::scene::ecs::{ChangeTickWindow, SystemParamAccess, SystemParamError};
use crate::scene::World;

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
        }
    };
}

tuple_system_param!(A);
tuple_system_param!(A, B);
tuple_system_param!(A, B, C);
tuple_system_param!(A, B, C, D);
