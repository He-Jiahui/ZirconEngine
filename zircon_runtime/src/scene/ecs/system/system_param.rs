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

    /// Releases state that is attached to a concrete `World` before a system
    /// is permanently retired or rebound. Stateless parameters keep the
    /// default implementation.
    fn retire_state(_world: &mut World, _state: &mut Self::State) {}

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

macro_rules! tuple_system_param_index {
    () => { 0usize };
    ($head:ident $(, $tail:ident)*) => {
        1usize + tuple_system_param_index!($($tail),*)
    };
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

macro_rules! init_tuple_system_param {
    (
        $world:ident,
        $access:ident;
        $(($param:ident, $state:ident)),+ $(,)?
    ) => {{
        init_tuple_system_param!(@next $world, $access; (); $(($param, $state)),+)
    }};
    (
        @next $world:ident,
        $access:ident;
        ($(($completed_param:ident, $completed_state:ident)),*);
        ($param:ident, $state:ident)
        $(, ($remaining_param:ident, $remaining_state:ident))* $(,)?
    ) => {{
        let mut $state = match $param::init_state($world, $access) {
            Ok(state) => state,
            Err(error) => {
                $($completed_param::retire_state($world, &mut $completed_state);)*
                return Err(error.in_tuple(
                    tuple_system_param_index!($($completed_param),*),
                    std::any::type_name::<$param>(),
                ));
            }
        };
        init_tuple_system_param!(
            @next $world,
            $access;
            ($(($completed_param, $completed_state),)* ($param, $state));
            $(($remaining_param, $remaining_state)),*
        )
    }};
    (
        @next $world:ident,
        $access:ident;
        ($(($param:ident, $state:ident)),*);
    ) => {
        Ok(($($state,)*))
    };
}

macro_rules! tuple_system_param {
    ($(($name:ident, $state:ident)),*) => {
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
                init_tuple_system_param!(world, access; $(($name, $state)),*)
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
            fn retire_state(world: &mut World, state: &mut Self::State) {
                let ($($name,)*) = state;
                $($name::retire_state(world, $name);)*
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

tuple_system_param!((A, state_a));
tuple_system_param!((A, state_a), (B, state_b));
tuple_system_param!((A, state_a), (B, state_b), (C, state_c));
tuple_system_param!((A, state_a), (B, state_b), (C, state_c), (D, state_d));
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k),
    (L, state_l)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k),
    (L, state_l),
    (M, state_m)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k),
    (L, state_l),
    (M, state_m),
    (N, state_n)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k),
    (L, state_l),
    (M, state_m),
    (N, state_n),
    (O, state_o)
);
tuple_system_param!(
    (A, state_a),
    (B, state_b),
    (C, state_c),
    (D, state_d),
    (E, state_e),
    (F, state_f),
    (G, state_g),
    (H, state_h),
    (I, state_i),
    (J, state_j),
    (K, state_k),
    (L, state_l),
    (M, state_m),
    (N, state_n),
    (O, state_o),
    (P, state_p)
);

#[cfg(test)]
#[path = "tests/system_param.rs"]
mod tests;
