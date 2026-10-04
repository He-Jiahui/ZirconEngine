use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

#[test]
fn param_set_commands_are_merged_and_applied_once() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Target".to_owned()), Health(10)))
        .unwrap();
    let mut state = SystemState::<ParamSet<(CommandsParam,)>>::new(&mut world).unwrap();

    state.run(&mut world, |mut set| {
        set.p0().entity(entity).insert(Marker);
    });
    assert!(world.get::<Marker>(entity).is_none());
    world.apply_deferred();
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));

    world.remove::<Marker>(entity).unwrap();
    state.run(&mut world, |_| {});
    world.apply_deferred();
    assert!(world.get::<Marker>(entity).is_none());
}

#[test]
fn nested_param_set_commands_survive_tuple_composition() {
    type Nested = (
        LocalParam<LocalCounter>,
        ParamSet<(ResParam<Score>, ParamSet<(CommandsParam,)>)>,
    );
    let mut world = World::empty();
    world.insert_resource(Score(7));
    let entity = world
        .spawn((Name("Target".to_owned()), Health(10)))
        .unwrap();
    let mut state = SystemState::<Nested>::new(&mut world).unwrap();

    state.run(&mut world, |(mut local, mut outer)| {
        local.0 += 1;
        assert_eq!(outer.p0().0, 7);
        outer.p1().p0().entity(entity).insert(Marker);
    });
    world.apply_deferred();
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
}

#[test]
fn param_set_commands_from_panicking_run_are_discarded() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Target".to_owned()), Health(10)))
        .unwrap();
    let mut state = SystemState::<ParamSet<(CommandsParam,)>>::new(&mut world).unwrap();

    let result = catch_unwind(AssertUnwindSafe(|| {
        state.run(&mut world, |mut set| {
            set.p0().entity(entity).insert(Marker);
            panic!("intentional command callback failure");
        });
    }));
    assert!(result.is_err());
    world.apply_deferred();
    assert!(world.get::<Marker>(entity).is_none());

    state.run(&mut world, |mut set| {
        set.p0().entity(entity).insert(Marker);
    });
    world.apply_deferred();
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
}

#[test]
fn nested_param_sets_reject_more_than_one_command_lane() {
    let mut world = World::empty();
    type TwoLanes = ParamSet<(ParamSet<(CommandsParam,)>, CommandsParam)>;
    assert_eq!(
        SystemState::<TwoLanes>::new(&mut world).unwrap_err(),
        SystemParamError::MultipleDeferredCommandParams,
    );
}
