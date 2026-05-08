use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, CommandsParam, Component, LocalParam, QueryState, ResMutParam, Resource,
    SystemState, With,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct Score(u32);

impl Resource for Score {}

#[derive(Default, Debug, PartialEq, Eq)]
struct LocalCounter(u32);

#[test]
fn commands_are_deferred_until_apply_deferred() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Queued".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(7));
        commands.spawn((Name("Spawned".to_string()), Health(3)));
        commands.insert_resource(Score(9));
    }

    assert!(world.get::<Health>(entity).is_none());
    assert!(world.get_resource::<Score>().is_none());
    assert_eq!(world.query::<&Health>().iter(&world).count(), 0);

    world.apply_deferred();

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(9)));
    assert_eq!(world.query::<&Health>().iter(&world).count(), 2);
}

#[test]
fn system_state_runs_query_resource_and_commands_params() {
    let mut world = World::empty();
    world.insert_resource(Score(1));
    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let enemy = world.spawn((Name("Enemy".to_string()), Health(4))).unwrap();

    type Params = (
        QueryState<&'static mut Health, With<Player>>,
        ResMutParam<Score>,
        CommandsParam,
    );
    let mut system = SystemState::<Params>::new(&mut world).unwrap();

    system.run(&mut world, |(mut health_query, mut score, mut commands)| {
        health_query.for_each_mut(|health| health.0 += 2);
        score.0 += 1;
        commands.insert(enemy, Marker);
    });

    assert_eq!(world.get::<Health>(player), Some(&Health(12)));
    assert_eq!(world.get::<Health>(enemy), Some(&Health(4)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(2)));
    assert!(world.get::<Marker>(enemy).is_none());

    world.apply_deferred();

    assert_eq!(world.get::<Marker>(enemy), Some(&Marker));
}

#[test]
fn added_and_changed_filters_use_system_run_windows() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();

    type AddedHealth = QueryState<(EntityId, &'static Health), Added<Health>>;
    let mut added_system = SystemState::<AddedHealth>::new(&mut world).unwrap();

    let first_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(first_added, vec![first]);

    let second_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert!(second_added.is_empty());

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let new_added = added_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(new_added, vec![second]);

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut changed_system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let baseline = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![first, second]);

    let unchanged = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert!(unchanged.is_empty());

    world.get_mut::<Health>(first).unwrap().0 += 5;
    let changed = changed_system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![first]);
}

#[test]
fn local_param_state_persists_between_system_runs() {
    let mut world = World::empty();
    let mut system = SystemState::<LocalParam<LocalCounter>>::new(&mut world).unwrap();

    let first = system.run(&mut world, |mut counter| {
        counter.0 += 1;
        counter.0
    });
    let second = system.run(&mut world, |mut counter| {
        counter.0 += 1;
        counter.0
    });

    assert_eq!(first, 1);
    assert_eq!(second, 2);
}
