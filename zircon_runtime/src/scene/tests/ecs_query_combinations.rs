use crate::scene::components::Name;
use crate::scene::ecs::{Changed, Component, QueryState, SystemState};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

fn compact_source(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join("")
}

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
    let changed = system.run(&mut world, |mut query| {
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
fn read_only_combination_candidates_use_single_scan() {
    let source = include_str!("../ecs/query/query_combinations_iter.rs");
    let constructor = source
        .split("pub(crate) fn new(")
        .nth(1)
        .and_then(|text| text.split("fn empty").next())
        .expect("read read-only combination constructor");

    assert!(constructor.contains("let mut matched_entities = Vec::new();"));
    assert!(constructor.contains("for entity in entities.iter().copied()"));
    assert!(
        constructor
            .contains("read_only_combination_candidate_matches::<D, F>(world, entity, ticks)")
    );
    assert!(constructor.contains("matched_entities.push(entity);"));
    assert!(constructor.contains("if matched_entities.len() < K"));
    assert!(!source.contains("fn read_only_combination_candidate_count"));
}

#[test]
fn cached_combination_candidates_use_compiled_archetype_plans() {
    let read_only_source = include_str!("../ecs/query/query_combinations_iter.rs");
    let mutable_source = include_str!("../ecs/query/query_combinations_mut_iter.rs");
    let read_only_cached_body = read_only_source
        .split("pub(crate) fn new_from_cached_plans")
        .nth(1)
        .and_then(|text| text.split("fn fetch_current").next())
        .expect("read cached read-only combination constructor");
    let mutable_cached_body = mutable_source
        .split("pub(crate) fn new_from_cached_plans")
        .nth(1)
        .and_then(|text| text.split("fn empty").next())
        .expect("read cached mutable combination constructor");

    assert!(read_only_cached_body.contains("world.stable_query_location_iter("));
    assert!(read_only_cached_body.contains("find_cached_archetype_plan(plans"));
    assert!(read_only_cached_body.contains("plan.write_component_locations("));
    let read_only_cached_compact = compact_source(read_only_cached_body);
    assert!(read_only_cached_compact.contains(
        "F::matches_component_locations(world,stable_location.stable_id,&component_locations,ticks"
    ));
    assert!(read_only_cached_body.contains("stable_locations.push(stable_location);"));
    assert!(!read_only_cached_body.contains("cached_component_locations"));

    assert!(mutable_cached_body.contains("world.stable_query_location_iter("));
    assert!(mutable_cached_body.contains("find_cached_archetype_plan(plans"));
    assert!(mutable_cached_body.contains("plan.write_component_locations("));
    let mutable_cached_compact = compact_source(mutable_cached_body);
    assert!(mutable_cached_compact.contains(
        "F::matches_component_locations(world,stable_location.stable_id,&component_locations,ticks"
    ));
    assert!(mutable_cached_body.contains("candidates.push(stable_location);"));
    assert!(!mutable_cached_body.contains("cached_component_locations"));
}
