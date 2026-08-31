use super::*;

#[test]
fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let unique_first = UniqueEntityArray::new([first]).unwrap();

    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many([first, marker_only])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first, first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_unique(unique_first)
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            UniqueEntityArray::new([first, first]),
            UniqueEntityArray::new([first, first]),
            UniqueEntityArray::new([first, first]),
        )
    });
    assert_eq!(
        baseline,
        (
            Ok([(first, 10), (first, 10)]),
            Err(QueryEntityError::QueryDoesNotMatch(marker_only)),
            Ok([(first, 10), (first, 10)]),
            Ok([(first, 10), (first, 10)]),
            Ok([(first, 10)]),
            Err(QueryEntityError::DuplicateEntity(first)),
            Err(QueryEntityError::DuplicateEntity(first)),
            Err(QueryEntityError::DuplicateEntity(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        unchanged,
        (
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
            Err(QueryEntityError::QueryDoesNotMatch(first)),
        )
    );
    assert_eq!(system.state().cache_rebuilds(), 1);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .get_many([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
            query
                .get_many_cached_direct([first])
                .map(|items| items.map(|(entity, health)| (entity, health.0))),
        )
    });
    assert_eq!(
        changed,
        (Ok([(first, 11)]), Ok([(first, 11)]), Ok([(first, 11)]),)
    );
}

#[test]
fn system_query_iter_many_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Marker)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    let requested = vec![marker_only, first, 999, first];
    let unique_first = UniqueEntityArray::new([first]).unwrap();
    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .iter_many(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_unique(unique_first)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_unique_cached(unique_first)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        baseline,
        (
            vec![(first, 10), (first, 10)],
            vec![(first, 10), (first, 10)],
            vec![(first, 10)],
            vec![(first, 10)]
        )
    );

    let unchanged = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert!(unchanged.0.is_empty());
    assert!(unchanged.1.is_empty());

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        (
            query
                .iter_many([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_cached([first])
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(changed, (vec![(first, 11)], vec![(first, 11)]));
}

#[test]
fn system_query_single_helpers_report_zero_one_many_matches() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<(EntityId, &'static Health), With<Player>>;
    let mut system = SystemState::<PlayerHealth>::new(&mut world).unwrap();

    let empty = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(empty, Err(QuerySingleError::NoEntities));

    let player = world
        .spawn((Name("Player".to_string()), Health(10), Player))
        .unwrap();
    let one = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(one, Ok((player, 10)));

    let cached = system.run(&mut world, |mut query| {
        query
            .single_cached()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached, Ok((player, 10)));

    let cached_direct = system.run(&mut world, |mut query| {
        query
            .single_cached_direct()
            .map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(cached_direct, Ok((player, 10)));

    world
        .spawn((Name("Ally".to_string()), Health(7), Player))
        .unwrap();
    let many = system.run(&mut world, |mut query| {
        query.single().map(|(entity, health)| (entity, health.0))
    });
    assert_eq!(many, Err(QuerySingleError::MultipleEntities));
}
