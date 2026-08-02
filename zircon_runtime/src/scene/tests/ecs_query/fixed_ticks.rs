use super::*;

#[test]
fn fixed_scene_components_are_queryable_through_m3_api() {
    let mut world = World::empty();
    let entity = world
        .spawn((
            Name("Renderable".to_string()),
            RenderLayerMask(0b0101),
            Health(8),
        ))
        .unwrap();

    let query = world.query_filtered::<(EntityId, &Name, &RenderLayerMask), With<Health>>();
    let rows = query.iter(&world).collect::<Vec<_>>();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, entity);
    assert_eq!(rows[0].1.0, "Renderable");
    assert_eq!(rows[0].2.0, 0b0101);
}

#[test]
fn ref_and_mut_query_items_report_change_ticks() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Tracked".to_string()), Health(10)))
        .unwrap();

    type ReadTracked = QueryState<(EntityId, Ref<'static, Health>)>;
    let mut read_system = crate::scene::ecs::SystemState::<ReadTracked>::new(&mut world).unwrap();
    let first = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.0, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(first, vec![(entity, 10, true, true)]);

    let second = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.is_added(), health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(second, vec![(entity, false, false)]);

    let mut write_system =
        crate::scene::ecs::SystemState::<QueryState<Mut<'static, Health>>>::new(&mut world)
            .unwrap();
    write_system.run(&mut world, |mut query| {
        query.for_each_mut(|mut health| {
            assert!(health.is_added());
            assert!(health.is_changed());
            health.0 += 5;
        });
    });

    let changed = read_system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.0, health.is_changed()))
            .collect::<Vec<_>>()
    });
    assert_eq!(changed, vec![(entity, 15, true)]);
}
