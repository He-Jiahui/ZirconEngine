use crate::scene::components::Name;
use crate::scene::ecs::{Changed, Component, QueryState, SystemState};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[test]
fn system_query_default_iter_reuses_persistent_cache_candidates() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let baseline = system.run(&mut world, |query| {
        let iter = query.iter();
        assert!(iter.uses_cached_component_locations());
        iter.map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(first, 10)]);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged_count = system.run(&mut world, |query| query.iter().count());
    assert_eq!(unchanged_count, 0);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let after_spawn = system.run(&mut world, |query| {
        let iter = query.iter();
        assert!(iter.uses_cached_component_locations());
        iter.map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_spawn, vec![(second, 20)]);
    assert_eq!(system.state().cache_rebuilds(), 2);
    assert_eq!(system.state().cached_entity_count(), 2);

    let marker_count = system.run(&mut world, |query| {
        query
            .iter_many([marker_only])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert!(marker_count.is_empty());
}

#[test]
fn query_cache_entity_index_rebuilds_and_preserves_requested_order() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut query = HealthQuery::new(&mut world);
    assert_eq!(query.cached_entity_index(first), Some(0));
    assert_eq!(query.cached_entity_index(second), Some(1));
    assert_eq!(query.cached_entity_index(marker_only), None);
    assert!(!query.iter(&world).uses_cached_component_locations());
    assert!(query.iter_cached(&world).uses_cached_component_locations());

    let requested = query
        .iter_many_cached_direct(&world, [second, first, marker_only, second])
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(requested, vec![(second, 20), (first, 10), (second, 20)]);

    world.remove::<Health>(first).unwrap();
    assert_eq!(query.count_cached(&world), 1);
    assert_eq!(query.cached_entity_index(first), None);
    assert_eq!(query.cached_entity_index(second), Some(0));
    assert!(!query.contains_cached(&world, first));
    assert!(query.contains_cached(&world, second));

    let after_remove = query
        .iter_many_cached(&world, [first, second])
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_remove, vec![(second, 20)]);
}
