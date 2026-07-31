use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::Hierarchy;
use crate::scene::{NodeKind, World};

#[test]
fn compiled_descendant_name_index_omits_root_and_preserves_hierarchy_order() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let arm = world.spawn_node(NodeKind::Mesh);
    let hand = world.spawn_node(NodeKind::Mesh);
    let sibling = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(arm, "Arm").unwrap();
    world.rename_node(hand, "Hand").unwrap();
    world.rename_node(sibling, "Sibling").unwrap();
    world.set_parent_checked(arm, Some(root)).unwrap();
    world.set_parent_checked(hand, Some(arm)).unwrap();
    world.set_parent_checked(sibling, Some(root)).unwrap();

    let binding = world.compile_descendant_name_index(root).unwrap();
    let entries = binding
        .entries()
        .iter()
        .map(|entry| (entry.entity(), entry.name()))
        .collect::<Vec<_>>();

    assert_eq!(
        entries,
        vec![(arm, "Arm"), (hand, "Hand"), (sibling, "Sibling")]
    );
    assert_eq!(binding.root(), root);
    assert_eq!(binding.generation(), world.scene_binding_generation(root));
    assert!(binding.is_current_for(&world));
}

#[test]
fn compiled_descendant_name_index_stays_current_after_transform_mutation() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(root)).unwrap();
    let binding = world.compile_descendant_name_index(root).unwrap();

    world
        .update_transform(child, Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)))
        .unwrap();

    assert!(binding.is_current_for(&world));
}

#[test]
fn compiled_descendant_name_index_ignores_unrelated_topology_and_name_changes() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Mesh);
    let unrelated_root = world.spawn_node(NodeKind::Empty);
    let unrelated_child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(root)).unwrap();
    world
        .set_parent_checked(unrelated_child, Some(unrelated_root))
        .unwrap();
    let binding = world.compile_descendant_name_index(root).unwrap();

    world
        .rename_node(unrelated_child, "Unrelated child")
        .unwrap();
    world.set_parent_checked(unrelated_child, None).unwrap();

    assert!(binding.is_current_for(&world));
}

#[test]
fn compiled_descendant_name_index_stales_for_name_hierarchy_and_topology_changes() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Mesh);
    let alternate_root = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(root)).unwrap();

    let before_name_change = world.compile_descendant_name_index(root).unwrap();
    world.rename_node(child, "Renamed child").unwrap();
    assert!(!before_name_change.is_current_for(&world));

    let before_reparent = world.compile_descendant_name_index(root).unwrap();
    world
        .set_parent_checked(child, Some(alternate_root))
        .unwrap();
    assert!(!before_reparent.is_current_for(&world));

    let before_spawn = world.compile_descendant_name_index(root).unwrap();
    let spawned = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(spawned, Some(root)).unwrap();
    assert!(!before_spawn.is_current_for(&world));

    let before_remove = world.compile_descendant_name_index(root).unwrap();
    assert!(world.remove_entity(spawned));
    assert!(!before_remove.is_current_for(&world));
}

#[test]
fn compiled_descendant_name_index_stales_when_root_id_is_reused() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(root)).unwrap();
    let root_record = world.node_record(root).unwrap();
    let binding = world.compile_descendant_name_index(root).unwrap();

    assert!(world.remove_entity(root));
    world.insert_node_record(root_record).unwrap();

    assert!(!binding.is_current_for(&world));
}

#[test]
fn raw_hierarchy_component_mutation_invalidates_cached_roots() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let alternate_root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(root)).unwrap();
    let root_binding = world.compile_descendant_name_index(root).unwrap();
    let alternate_binding = world.compile_descendant_name_index(alternate_root).unwrap();

    world.get_mut::<Hierarchy>(child).unwrap().parent = Some(alternate_root);

    assert!(!root_binding.is_current_for(&world));
    assert!(!alternate_binding.is_current_for(&world));
}

#[test]
fn compiled_scene_property_target_interns_ids_and_stales_for_its_path_changes() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    let unrelated_root = world.spawn_node(NodeKind::Empty);
    let unrelated_child = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.rename_node(unrelated_root, "Unrelated").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .set_parent_checked(unrelated_child, Some(unrelated_root))
        .unwrap();

    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let first = world
        .compile_scene_property_target(&entity_path, &property_path)
        .unwrap();
    let second = world
        .compile_scene_property_target(&entity_path, &property_path)
        .unwrap();

    assert_eq!(first.entity(), hero);
    assert_eq!(first.path_id(), second.path_id());
    assert_eq!(first.component_field_id(), second.component_field_id());
    assert!(first.is_current_for(&world));

    world
        .update_transform(hero, Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)))
        .unwrap();
    world
        .rename_node(unrelated_child, "Still unrelated")
        .unwrap();
    assert!(first.is_current_for(&world));

    world.rename_node(hero, "Renamed hero").unwrap();
    assert!(!first.is_current_for(&world));
}

#[test]
fn compiled_transform_property_reads_without_path_dispatch_and_requires_rebind_after_name_change() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .update_transform(hero, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();

    let accessor = world
        .compile_transform_property_target(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation").unwrap(),
        )
        .unwrap();

    assert_eq!(
        world.read_compiled_transform_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Vec3([
            1.0, 2.0, 3.0,
        ]))
    );

    world
        .update_transform(hero, Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)))
        .unwrap();
    assert_eq!(
        world.read_compiled_transform_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Vec3([
            4.0, 5.0, 6.0,
        ]))
    );

    world.rename_node(hero, "Renamed hero").unwrap();
    assert_eq!(world.read_compiled_transform_property(&accessor), None);
}

#[test]
fn compiled_transform_property_writes_without_string_dispatch() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    let accessor = world
        .compile_transform_property_target(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
        )
        .unwrap();

    assert!(
        world
            .write_compiled_transform_property(
                &accessor,
                crate::core::framework::scene::ScenePropertyValue::Scalar(7.0),
            )
            .unwrap()
    );
    assert_eq!(
        world.read_compiled_transform_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            7.0
        ))
    );
    assert!(
        !world
            .write_compiled_transform_property(
                &accessor,
                crate::core::framework::scene::ScenePropertyValue::Scalar(7.0),
            )
            .unwrap()
    );
}

#[test]
fn compiled_scene_property_target_stales_when_root_id_is_reused() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    world.rename_node(root, "Root").unwrap();
    let root_record = world.node_record(root).unwrap();
    let entity_path = EntityPath::parse("Root").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let target = world
        .compile_scene_property_target(&entity_path, &property_path)
        .unwrap();

    assert!(world.remove_entity(root));
    world.insert_node_record(root_record).unwrap();

    assert!(!target.is_current_for(&world));
}
