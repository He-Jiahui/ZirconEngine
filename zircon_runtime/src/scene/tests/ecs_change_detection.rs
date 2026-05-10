use crate::scene::components::Name;
use crate::scene::ecs::{Changed, Component, QueryState, RemovedComponentsParam, SystemState};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[test]
fn changed_filter_includes_newly_added_components() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Changed".to_string()), Health(1)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let changed = system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });

    assert_eq!(changed, vec![entity]);
}

#[test]
fn removed_components_tracks_recursive_despawn() {
    let mut world = World::empty();
    let parent = world
        .spawn((Name("Parent".to_string()), Health(1)))
        .unwrap();
    let child = world.spawn((Name("Child".to_string()), Health(2))).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    type RemovedHealth = RemovedComponentsParam<Health>;
    let mut system = SystemState::<RemovedHealth>::new(&mut world).unwrap();
    assert!(system
        .run(&mut world, |mut removed| removed.read().collect::<Vec<_>>())
        .is_empty());

    world.remove_entity_recursive(parent);

    let removed = system.run(&mut world, |mut removed| removed.read().collect::<Vec<_>>());
    assert_eq!(removed, vec![child, parent]);
}
