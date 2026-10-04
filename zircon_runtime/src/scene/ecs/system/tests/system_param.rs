use crate::scene::ecs::{ResMutParam, ResParam, Resource, SystemParamError, SystemState};
use crate::scene::World;

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
    .err()
    .expect("the sixteenth parameter must report its conflicting tuple position");

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
