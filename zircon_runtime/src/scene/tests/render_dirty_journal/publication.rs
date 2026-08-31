use super::*;

#[test]
fn render_dirty_publication_is_world_bound_and_reused_on_stable_frames() {
    let mut first = World::empty();
    first.spawn((RenderValue(1),)).unwrap();
    publish_render_dirty_journal(&mut first);
    let first_publication = first.render_dirty_entity_journal();

    publish_render_dirty_journal(&mut first);
    let stable_publication = first.render_dirty_entity_journal();

    let mut second = World::empty();
    second.spawn((RenderValue(1),)).unwrap();
    publish_render_dirty_journal(&mut second);
    let second_publication = second.render_dirty_entity_journal();

    assert!(Arc::ptr_eq(&first_publication, &stable_publication));
    assert_eq!(first_publication.generation(), 1);
    assert!(first_publication.all_entities());
    assert_ne!(first_publication.world(), second_publication.world());
}

#[test]
fn render_dirty_publication_sorts_deduplicates_and_preserves_removed_entities() {
    let mut world = World::empty();
    let first = world.spawn((RenderValue(1),)).unwrap();
    let second = world.spawn((RenderValue(2),)).unwrap();
    publish_render_dirty_journal(&mut world);

    world.get_mut::<RenderValue>(second).unwrap().0 += 1;
    world.get_mut::<RenderValue>(first).unwrap().0 += 1;
    world.get_mut::<RenderValue>(second).unwrap().0 += 1;
    world.remove::<RenderValue>(first).unwrap();
    publish_render_dirty_journal(&mut world);

    let publication = world.render_dirty_entity_journal();
    assert_eq!(publication.generation(), 2);
    assert!(!publication.all_entities());
    assert_eq!(publication.entities(), &[first, second]);
}

#[test]
fn render_transform_propagation_publishes_every_affected_descendant() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube).unwrap();
    let child = world.spawn_node(NodeKind::Mesh).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    publish_render_dirty_journal(&mut world);

    world
        .insert(
            parent,
            LocalTransform {
                transform: Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            },
        )
        .unwrap();
    publish_render_dirty_journal(&mut world);

    let publication = world.render_dirty_entity_journal();
    assert_eq!(publication.entities(), &[parent, child]);
}
