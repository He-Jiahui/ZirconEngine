use super::*;

#[test]
fn system_query_iter_many_mut_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Player)).unwrap();

    type ChangedHealth = QueryState<&'static mut Health, Changed<Health>>;
    let mut system = crate::scene::ecs::SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = vec![marker_only, first, 999, first];

    let baseline = system.run(&mut world, |mut query| {
        let mut iter = query.iter_many_mut(&requested);
        let mut seen = Vec::new();
        while let Some(health) = iter.fetch_next() {
            seen.push(health.0);
            health.0 += 1;
        }
        seen
    });
    assert_eq!(baseline, vec![10, 11]);
    assert_eq!(world.get::<Health>(first), Some(&Health(12)));

    let unchanged = system.run(&mut world, |mut query| {
        query.iter_many_mut([first]).fetch_next().is_none()
    });
    assert!(unchanged);

    world.get_mut::<Health>(first).unwrap().0 = 20;
    let changed = system.run(&mut world, |mut query| {
        let mut iter = query.iter_many_mut([first]);
        let health = iter.fetch_next().unwrap();
        health.0 += 1;
        health.0
    });
    assert_eq!(changed, 21);
}

#[test]
fn system_query_iter_many_cached_direct_preserves_order_duplicates_and_run_window_filters() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First".to_string()), Health(10)))
        .unwrap();
    let marker_only = world.spawn((Name("Marker".to_string()), Player)).unwrap();

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = crate::scene::ecs::SystemState::<ChangedHealth>::new(&mut world).unwrap();
    let requested = vec![marker_only, first, 999, first];

    let unique_first = UniqueEntityArray::new([first]).unwrap();
    let baseline = system.run(&mut world, |mut query| {
        (
            query
                .iter_many_cached_direct(&requested)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
            query
                .iter_many_unique_cached_direct(unique_first)
                .map(|(entity, health)| (entity, health.0))
                .collect::<Vec<_>>(),
        )
    });
    assert_eq!(
        baseline,
        (vec![(first, 10), (first, 10)], vec![(first, 10)])
    );

    let unchanged = system.run(&mut world, |mut query| {
        query
            .iter_many_cached_direct([first])
            .collect::<Vec<_>>()
            .is_empty()
    });
    assert!(unchanged);

    world.get_mut::<Health>(first).unwrap().0 = 11;
    let changed = system.run(&mut world, |mut query| {
        query
            .iter_many_cached_direct([first])
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(first, 11)]);
}
