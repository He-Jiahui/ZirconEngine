use crate::scene::components::{MeshRenderer, Name};
use crate::scene::ecs::{Changed, Component, QueryState, SystemState};
use crate::scene::{EntityId, NodeKind, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct UnrelatedMarker;

impl Component for UnrelatedMarker {}

type NameQuery = QueryState<(EntityId, &'static Name)>;

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

    let baseline = system.run(&mut world, |mut query| {
        let iter = query.iter();
        assert!(iter.uses_compiled_archetype_plans());
        iter.map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(first, 10)]);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged_count = system.run(&mut world, |mut query| query.iter().count());
    assert_eq!(unchanged_count, 0);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let after_spawn = system.run(&mut world, |mut query| {
        let iter = query.iter();
        assert!(iter.uses_compiled_archetype_plans());
        iter.map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_spawn, vec![(second, 20)]);
    assert_eq!(system.state().cache_rebuilds(), 1);
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
fn query_cache_plans_preserve_requested_order_without_entity_index_projection() {
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
    assert!(query.contains_cached(&world, first));
    assert!(query.contains_cached(&world, second));
    assert!(!query.contains_cached(&world, marker_only));
    assert!(!query.iter(&world).uses_compiled_archetype_plans());
    assert!(query.iter_cached(&world).uses_compiled_archetype_plans());

    let requested = query
        .iter_many_cached_direct(&world, [second, first, marker_only, second])
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(requested, vec![(second, 20), (first, 10), (second, 20)]);

    world.remove::<Health>(first).unwrap();
    assert_eq!(query.count_cached(&world), 1);
    assert!(!query.contains_cached(&world, first));
    assert!(query.contains_cached(&world, second));

    let after_remove = query
        .iter_many_cached(&world, [first, second])
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_remove, vec![(second, 20)]);
}

#[test]
fn cached_query_ignores_membership_changes_in_unmatched_existing_archetypes() {
    let mut world = World::empty();
    let matched = world
        .spawn((Name("Matched".to_string()), Health(10)))
        .unwrap();
    let unmatched = world
        .spawn((Name("Unmatched".to_string()), Marker))
        .unwrap();
    world
        .spawn((
            Name("Target archetype".to_string()),
            Marker,
            UnrelatedMarker,
        ))
        .unwrap();

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut query = HealthQuery::new(&mut world);
    assert_eq!(query.cache_rebuilds(), 1);

    world.insert(unmatched, UnrelatedMarker).unwrap();

    let cached = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(cached, vec![(matched, 10)]);
    assert_eq!(query.cache_rebuilds(), 1);
}

#[test]
fn cached_query_compiles_only_new_archetypes_that_match_its_access() {
    let mut world = World::empty();
    let matched = world
        .spawn((Name("Matched".to_string()), Health(10)))
        .unwrap();

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut query = HealthQuery::new(&mut world);
    let initial_generation = query.cached_archetype_generation();
    assert_eq!(query.cache_rebuilds(), 1);

    world
        .spawn((
            Name("New unmatched archetype".to_string()),
            Marker,
            UnrelatedMarker,
        ))
        .unwrap();
    let after_unmatched = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_unmatched, vec![(matched, 10)]);
    assert_eq!(query.cache_rebuilds(), 1);
    assert!(query.cached_archetype_generation() > initial_generation);

    let second = world
        .spawn((
            Name("New matching archetype".to_string()),
            Health(20),
            UnrelatedMarker,
        ))
        .unwrap();
    let after_matching = query
        .iter_cached(&world)
        .map(|(entity, health)| (entity, health.0))
        .collect::<Vec<_>>();
    assert_eq!(after_matching, vec![(matched, 10), (second, 20)]);
    assert_eq!(query.cache_rebuilds(), 2);
}

#[test]
fn cached_name_query_keeps_stable_world_order_across_moves_clone_and_serde() {
    let mut world = World::empty();
    let first_mesh = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let second_mesh = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let camera = world
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");

    world
        .remove::<MeshRenderer>(first_mesh)
        .expect("first mesh renderer should be removable");

    assert_cached_name_order(&mut world, &[first_mesh, second_mesh, camera]);

    let mut cloned = world.clone();
    assert_cached_name_order(&mut cloned, &[first_mesh, second_mesh, camera]);

    let encoded = serde_json::to_string(&world).expect("world should serialize");
    let mut restored: World = serde_json::from_str(&encoded).expect("world should deserialize");
    assert_cached_name_order(&mut restored, &[first_mesh, second_mesh, camera]);

    fn assert_cached_name_order(world: &mut World, expected: &[EntityId]) {
        let mut query = NameQuery::new(world);
        let actual = query
            .iter_cached(world)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}
