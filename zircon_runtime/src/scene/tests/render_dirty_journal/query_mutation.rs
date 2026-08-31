use super::*;

#[test]
fn render_lazy_mut_query_records_only_entities_that_are_actually_changed() {
    let mut world = World::empty();
    let first = world.spawn((RenderValue(1),)).unwrap();
    let second = world.spawn((RenderValue(2),)).unwrap();
    publish_render_dirty_journal(&mut world);
    let stable_publication = world.render_dirty_entity_journal();
    let stable_world_generation = world.world_generation();

    let mut query = world.query::<Mut<'static, RenderValue>>();
    for value in query.iter_mut(&mut world) {
        let _ = value.0;
    }
    assert_eq!(world.world_generation(), stable_world_generation);
    publish_render_dirty_journal(&mut world);
    assert!(Arc::ptr_eq(
        &stable_publication,
        &world.render_dirty_entity_journal()
    ));

    {
        let mut value = query.get_mut(&mut world, second).unwrap();
        value.0 += 1;
        value.set_changed();
    }
    assert_eq!(world.world_generation(), stable_world_generation + 1);
    assert!(world.has_pending_scene_systems());

    let mut cloned = world.clone();
    assert_eq!(cloned.world_generation(), world.world_generation());
    publish_render_dirty_journal(&mut cloned);
    assert_ne!(
        cloned.render_dirty_entity_journal().world(),
        stable_publication.world()
    );

    publish_render_dirty_journal(&mut world);

    let publication = world.render_dirty_entity_journal();
    assert_eq!(publication.entities(), &[second]);
    assert_eq!(
        publication.source_world_generation(),
        world.world_generation()
    );
    assert_eq!(publication.source_change_tick(), world.read_change_tick());
    assert_eq!(world.get::<RenderValue>(first), Some(&RenderValue(1)));
    assert_eq!(world.get::<RenderValue>(second), Some(&RenderValue(3)));
}

#[test]
fn render_lazy_transform_query_drives_derived_state_before_publication() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube).unwrap();
    let child = world.spawn_node(NodeKind::Mesh).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    publish_render_dirty_journal(&mut world);

    let mut query = world.query::<Mut<'static, LocalTransform>>();
    query
        .get_mut(&mut world, parent)
        .unwrap()
        .transform
        .translation
        .x = 7.0;
    publish_render_dirty_journal(&mut world);

    assert_eq!(world.world_transform(child).unwrap().translation.x, 7.0);
    assert_eq!(
        world.render_dirty_entity_journal().entities(),
        &[parent, child]
    );
}
