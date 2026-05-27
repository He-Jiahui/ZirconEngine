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
    assert_eq!(query.iter_combinations::<4>(&world).count(), 0);
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
