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
fn query_state_iter_combinations_returns_unique_read_only_groups() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let third = world
        .spawn((Name("Third".to_string()), Health(30)))
        .unwrap();
    world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<(EntityId, &Health)>();
    let pairs = query
        .iter_combinations::<2>(&world)
        .map(|[(left, left_health), (right, right_health)]| {
            (left, left_health.0, right, right_health.0)
        })
        .collect::<Vec<_>>();
    let cached_pairs = query
        .iter_combinations_cached::<2>(&world)
        .map(|[(left, left_health), (right, right_health)]| {
            (left, left_health.0, right, right_health.0)
        })
        .collect::<Vec<_>>();
    let triples = query
        .iter_combinations::<3>(&world)
        .map(|items| items.map(|(entity, health)| (entity, health.0)))
        .collect::<Vec<_>>();

    assert_eq!(
        pairs,
        vec![
            (first, 10, second, 20),
            (first, 10, third, 30),
            (second, 20, third, 30)
        ]
    );
    assert_eq!(cached_pairs, pairs);
    assert_eq!(triples, vec![[(first, 10), (second, 20), (third, 30)]]);
    let oversized = query.iter_combinations::<4>(&world);
    assert_eq!(oversized.size_hint(), (0, Some(0)));
    assert_eq!(oversized.count(), 0);
    let cached_oversized = query.iter_combinations_cached::<4>(&world);
    assert_eq!(cached_oversized.size_hint(), (0, Some(0)));
    assert_eq!(cached_oversized.count(), 0);
}

#[test]
fn system_query_iter_combinations_uses_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let third = world
        .spawn((Name("Third".to_string()), Health(30)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .iter_combinations::<2>()
                .map(|items| items.map(|(entity, health)| (entity, health.0)))
                .collect::<Vec<_>>(),
            query
                .iter_combinations_cached::<2>()
                .map(|items| items.map(|(entity, health)| (entity, health.0)))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        baseline,
        (
            vec![
                [(first, 10), (second, 20)],
                [(first, 10), (third, 30)],
                [(second, 20), (third, 30)]
            ],
            vec![
                [(first, 10), (second, 20)],
                [(first, 10), (third, 30)],
                [(second, 20), (third, 30)]
            ],
        )
    );

    let unchanged = system.run(&mut world, |mut query| {
        (
            query.iter_combinations::<2>().count(),
            query.iter_combinations_cached::<2>().count(),
        )
    });
    assert_eq!(unchanged, (0, 0));

    world.get_mut::<Health>(first).unwrap().0 += 1;
    world.get_mut::<Health>(third).unwrap().0 += 1;
    let changed = system.run(&mut world, |query| {
        query
            .iter_combinations::<2>()
            .map(|items| items.map(|(entity, health)| (entity, health.0)))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![[(first, 11), (third, 31)]]);
}

#[test]
fn query_state_iter_combinations_mut_fetch_next_mutates_unique_groups() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let third = world
        .spawn((Name("Third".to_string()), Health(30)))
        .unwrap();

    let mut query = world.query::<&mut Health>();
    let mut combinations = query.iter_combinations_mut::<2>(&mut world);
    assert_eq!(combinations.size_hint(), (3, Some(3)));

    let mut visited = 0;
    while let Some([left, right]) = combinations.fetch_next() {
        left.0 += 1;
        right.0 += 10;
        visited += 1;
    }
    assert_eq!(visited, 3);
    drop(combinations);

    assert_eq!(world.get::<Health>(first), Some(&Health(12)));
    assert_eq!(world.get::<Health>(second), Some(&Health(31)));
    assert_eq!(world.get::<Health>(third), Some(&Health(50)));

    let mut oversized = query.iter_combinations_mut::<4>(&mut world);
    assert_eq!(oversized.size_hint(), (0, Some(0)));
    assert!(oversized.fetch_next().is_none());
}

#[test]
fn system_query_iter_combinations_mut_fetch_next_mutates_unique_groups() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let third = world
        .spawn((Name("Third".to_string()), Health(30)))
        .unwrap();

    type HealthQuery = QueryState<&'static mut Health>;
    let mut system = SystemState::<HealthQuery>::new(&mut world).unwrap();

    let visited = system.run(&mut world, |mut query| {
        let mut combinations = query.iter_combinations_mut::<2>();
        assert_eq!(combinations.size_hint(), (3, Some(3)));

        let mut visited = 0;
        while let Some([left, right]) = combinations.fetch_next() {
            left.0 += 1;
            right.0 += 10;
            visited += 1;
        }
        visited
    });
    assert_eq!(visited, 3);

    assert_eq!(world.get::<Health>(first), Some(&Health(12)));
    assert_eq!(world.get::<Health>(second), Some(&Health(31)));
    assert_eq!(world.get::<Health>(third), Some(&Health(50)));
}

#[test]
fn read_only_combination_candidate_count_uses_direct_scan() {
    let source = include_str!("../ecs/query/query_combinations_iter.rs");
    let count_body = source
        .split("fn read_only_combination_candidate_count")
        .nth(1)
        .and_then(|text| {
            text.split("fn read_only_combination_candidate_matches")
                .next()
        })
        .expect("read read-only combination candidate counter");

    assert!(count_body.contains("let mut count = 0_usize;"));
    assert!(count_body.contains("for entity in entities.iter().copied()"));
    assert!(count_body
        .contains("read_only_combination_candidate_matches::<D, F>(world, entity, ticks)"));
    assert!(count_body.contains("count += 1;"));
    assert!(!count_body.contains(".filter("));
    assert!(!count_body.contains(".count()"));
}

#[test]
fn cached_combination_candidate_indices_use_direct_index_scans() {
    let read_only_source = include_str!("../ecs/query/query_combinations_iter.rs");
    let mutable_source = include_str!("../ecs/query/query_combinations_mut_iter.rs");
    let read_only_cached_body = read_only_source
        .split("pub(crate) fn new_from_cached_entities")
        .nth(1)
        .and_then(|text| text.split("fn fetch_current").next())
        .expect("read cached read-only combination constructor");
    let mutable_cached_body = mutable_source
        .split("pub(crate) fn new_from_cached_entities")
        .nth(1)
        .and_then(|text| text.split("fn empty").next())
        .expect("read cached mutable combination constructor");

    assert!(read_only_cached_body.contains("let mut index = 0_usize;"));
    assert!(read_only_cached_body.contains("while index < entities.len()"));
    assert!(read_only_cached_body.contains("let entity = entities[index];"));
    assert!(read_only_cached_body.contains("stable_locations.get(index).is_some()"));
    assert!(read_only_cached_body.contains("cached_query_component_locations("));
    assert!(read_only_cached_body.contains(
        "F::matches_component_locations(world, entity, entity_component_locations, ticks)"
    ));
    assert!(read_only_cached_body.contains("cache_indices.push(index);"));
    assert!(read_only_cached_body.contains("index += 1;"));
    assert!(!read_only_cached_body.contains("entities.iter().copied().enumerate()"));

    assert!(mutable_cached_body.contains("let mut index = 0_usize;"));
    assert!(mutable_cached_body.contains("while index < entities.len()"));
    assert!(mutable_cached_body.contains("let entity = entities[index];"));
    assert!(mutable_cached_body.contains("cached_query_component_locations("));
    assert!(mutable_cached_body
        .contains("F::matches_component_locations(world, entity, component_locations, ticks)"));
    assert!(mutable_cached_body.contains("cache_indices.push(index);"));
    assert!(mutable_cached_body.contains("index += 1;"));
    assert!(!mutable_cached_body.contains("entities.iter().copied().enumerate()"));
}
