use crate::scene::components::Name;
use crate::scene::ecs::{
    Changed, Component, Mut, QueryEntityError, QueryState, SystemState, UniqueEntityArray,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[test]
fn mutable_query_data_supports_read_only_projection_helpers() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    assert_eq!(
        query
            .iter(&world)
            .map(|health| health.0)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(query.get(&world, first).map(|health| health.0), Ok(10));
    assert_eq!(
        query
            .iter_cached_direct(&world)
            .map(|health| health.0)
            .collect::<Vec<_>>(),
        vec![10, 20]
    );
    assert_eq!(
        query.get_cached_direct(&world, marker_only).map(|_| ()),
        Err(QueryEntityError::QueryDoesNotMatch(marker_only))
    );

    query.iter_mut(&mut world).for_each(|health| health.0 += 1);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));

    let mut tick_query = world.query::<Mut<'static, Health>>();
    assert_eq!(
        tick_query
            .iter_cached_direct(&world)
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>(),
        vec![(11, true, true), (21, true, true)]
    );
}

#[test]
fn query_state_iter_mut_mutates_all_cached_matches_once() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let seen = query
        .iter_mut(&mut world)
        .map(|health| {
            let before = health.0;
            health.0 += 1;
            before
        })
        .collect::<Vec<_>>();

    assert_eq!(seen, vec![10, 20]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
    assert_eq!(world.get::<Health>(marker_only), None);
}

#[test]
fn query_state_iter_many_unique_mut_mutates_unique_targets_and_skips_mismatches() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let requested = UniqueEntityArray::new([marker_only, first, 999, second]).unwrap();
    let seen = query
        .iter_many_unique_mut(&mut world, requested)
        .map(|health| {
            let before = health.0;
            health.0 += 1;
            before
        })
        .collect::<Vec<_>>();

    assert_eq!(seen, vec![10, 20]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
    assert_eq!(
        UniqueEntityArray::new([first, first]),
        Err(QueryEntityError::DuplicateEntity(first))
    );
}

#[test]
fn query_state_get_many_unique_mut_mutates_all_targets_or_reports_mismatch() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    let mut query = world.query::<&mut Health>();
    let [left, right] = query
        .get_many_unique_mut(&mut world, UniqueEntityArray::new([first, second]).unwrap())
        .unwrap();
    left.0 += 1;
    right.0 += 10;
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(30)));

    assert_eq!(
        query
            .get_many_unique_mut(
                &mut world,
                UniqueEntityArray::new([first, marker_only]).unwrap(),
            )
            .map(|_| ()),
        Err(QueryEntityError::QueryDoesNotMatch(marker_only))
    );
}

#[test]
fn system_mutable_query_data_read_only_projection_uses_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |query| {
        query.iter().map(|health| health.0).collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![10, 20]);

    let unchanged = system.run(&mut world, |query| query.iter().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |query| {
        query.iter().map(|health| health.0).collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![21]);
    assert_eq!(world.get::<Health>(first), Some(&Health(10)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
}

#[test]
fn system_mut_tick_wrapper_cached_direct_projection_preserves_ref_ticks() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<Mut<'static, Health>, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(10, true, true), (20, true, true)]);

    let unchanged = system.run(&mut world, |mut query| query.iter_cached_direct().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|health| (health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(21, false, true)]);
    assert_eq!(world.get::<Health>(first), Some(&Health(10)));
    assert_eq!(world.get::<Health>(second), Some(&Health(21)));
}

#[test]
fn system_query_iter_mut_keeps_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_mut()
            .map(|health| {
                health.0 += 1;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![11, 21]);

    let unchanged = system.run(&mut world, |mut query| query.iter_mut().count());
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_mut()
            .map(|health| {
                health.0 += 10;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![32]);
    assert_eq!(world.get::<Health>(first), Some(&Health(11)));
    assert_eq!(world.get::<Health>(second), Some(&Health(32)));
}

#[test]
fn system_query_iter_many_unique_mut_is_iterator_and_keeps_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = UniqueEntityArray::new([marker_only, first, 999, second]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_many_unique_mut(requested)
            .map(|health| {
                health.0 += 1;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![11, 21]);

    let unchanged = system.run(&mut world, |mut query| {
        query.iter_many_unique_mut(requested).count()
    });
    assert_eq!(unchanged, 0);

    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_many_unique_mut(requested)
            .map(|health| {
                health.0 += 10;
                health.0
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![32]);
}

#[test]
fn system_query_get_many_unique_mut_uses_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let second = world
        .spawn((Name("Second".to_string()), Health(20)))
        .unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = UniqueEntityArray::new([first, second]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        let [left, right] = query.get_many_unique_mut(requested).unwrap();
        left.0 += 1;
        right.0 += 10;
        [left.0, right.0]
    });
    assert_eq!(baseline, [11, 30]);

    let unchanged = system.run(&mut world, |mut query| {
        query.get_many_unique_mut(requested).map(|_| ())
    });
    assert_eq!(unchanged, Err(QueryEntityError::QueryDoesNotMatch(first)));

    world.get_mut::<Health>(first).unwrap().0 += 1;
    world.get_mut::<Health>(second).unwrap().0 += 1;
    let changed = system.run(&mut world, |mut query| {
        let [left, right] = query.get_many_unique_mut(requested).unwrap();
        left.0 += 10;
        right.0 += 100;
        [left.0, right.0]
    });
    assert_eq!(changed, [22, 131]);
}
