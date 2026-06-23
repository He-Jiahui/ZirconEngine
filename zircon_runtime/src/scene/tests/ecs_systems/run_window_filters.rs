use super::*;

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
fn system_query_cached_direct_rechecks_run_window_filters() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let first = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(first, vec![(entity, 10)]);

    let second = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert!(second.is_empty());

    world.get_mut::<Health>(entity).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_cached_direct()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(entity, 11)]);
}

#[test]
fn system_query_iter_cached_reuses_state_and_rechecks_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let baseline = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(baseline, vec![(first, 10)]);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert!(unchanged.is_empty());
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .single_cached()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(changed, Ok((first, 11)));
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let after_spawn = system.run(&mut world, |mut query| {
        query
            .iter_cached()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_spawn, vec![(second, 1)]);
    assert_eq!(system.state().cache_rebuilds(), 2);
}

#[test]
fn system_query_count_and_empty_helpers_reuse_cache_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let baseline = system.run(&mut world, |mut query| {
        (
            query.count(),
            query.is_empty(),
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains(first),
            query.contains(marker_only),
            query.get(first).map(|(entity, health)| (entity, health.0)),
            query
                .get(marker_only)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached(first),
            query.contains_cached(marker_only),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached(marker_only)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query.contains_cached_direct(marker_only),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached_direct(marker_only)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        (baseline.0, baseline.1, baseline.2, baseline.3, baseline.4, baseline.5),
        (1, false, 1, false, 1, false)
    );
    assert_eq!(
        (baseline.6, baseline.7, baseline.8, baseline.9),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(
        (baseline.10, baseline.11, baseline.12, baseline.13),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(
        (baseline.14, baseline.15, baseline.16, baseline.17),
        (
            true,
            false,
            Ok((first, 10)),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        (
            query.count(),
            query.is_empty(),
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains(first),
            query.get(first).map(|(entity, health)| (entity, health.0)),
            query.contains_cached(first),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        unchanged,
        (
            0,
            true,
            0,
            true,
            0,
            true,
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            false,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains_cached(first),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        changed,
        (
            1,
            false,
            1,
            false,
            true,
            Ok((first, 11)),
            true,
            Ok((first, 11))
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let second = world
        .spawn((Name("Second".to_string()), Health(1)))
        .unwrap();
    let after_spawn = system.run(&mut world, |mut query| {
        (
            query.count_cached(),
            query.is_empty_cached(),
            query.count_cached_direct(),
            query.is_empty_cached_direct(),
            query.contains_cached(first),
            query.contains_cached(second),
            query
                .get_cached(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached(second)
                .map(|(entity, health)| (entity, health.0)),
            query.contains_cached_direct(first),
            query.contains_cached_direct(second),
            query
                .get_cached_direct(first)
                .map(|(entity, health)| (entity, health.0)),
            query
                .get_cached_direct(second)
                .map(|(entity, health)| (entity, health.0)),
        )
    });
    assert_eq!(
        after_spawn,
        (
            1,
            false,
            1,
            false,
            false,
            true,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Ok((second, 1)),
            false,
            true,
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Ok((second, 1)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 2);
}
