use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, SystemParamAccess, SystemParamError, WorkerCommandBuffer,
};

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
mod tests {
    use crate::scene::World;
    use crate::scene::ecs::{ResMutParam, ResParam, Resource, SystemParamError, SystemState};

    struct TupleResource;

    impl Resource for TupleResource {}

    #[test]
    fn tuple_system_param_supports_sixteen_parameters() {
        let mut world = World::empty();
        let mut state = SystemState::<(
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
        )>::new(&mut world);

        let parameter_count = state
            .as_mut()
            .expect("sixteen parameters must initialize through the shared tuple macro")
            .run_without_world(
                |(
                    parameter_0,
                    parameter_1,
                    parameter_2,
                    parameter_3,
                    parameter_4,
                    parameter_5,
                    parameter_6,
                    parameter_7,
                    parameter_8,
                    parameter_9,
                    parameter_10,
                    parameter_11,
                    parameter_12,
                    parameter_13,
                    parameter_14,
                    parameter_15,
                )| {
                    let _ = (
                        parameter_0,
                        parameter_1,
                        parameter_2,
                        parameter_3,
                        parameter_4,
                        parameter_5,
                        parameter_6,
                        parameter_7,
                        parameter_8,
                        parameter_9,
                        parameter_10,
                        parameter_11,
                        parameter_12,
                        parameter_13,
                        parameter_14,
                        parameter_15,
                    );
                    16
                },
            );

        assert_eq!(parameter_count, 16);
    }

    #[test]
    fn tuple_system_param_reports_the_sixteenth_conflicting_parameter() {
        let mut world = World::empty();
        world.insert_resource(TupleResource);

        let error = SystemState::<(
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            ResParam<TupleResource>,
            ResMutParam<TupleResource>,
        )>::new(&mut world)
        .expect_err("the sixteenth parameter must report its conflicting tuple position");

        assert_eq!(
            error,
            SystemParamError::TupleElement {
                index: 15,
                parameter_type: std::any::type_name::<ResMutParam<TupleResource>>(),
                source: Box::new(SystemParamError::ConflictingResourceAccess {
                    resource_id: world.resource_id::<TupleResource>(),
                }),
            }
        );
    }
}
