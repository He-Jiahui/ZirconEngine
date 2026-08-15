use crate::core::framework::scene::{ComponentPropertyPath, ComponentTypeDescriptor, EntityPath};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{Hierarchy, MeshRenderer};
use crate::scene::{NodeKind, World};
use serde_json::json;

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
    world.remove_entity(spawned).unwrap();
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

    world.remove_entity(root).unwrap();
    world.insert_node_record(root_record).unwrap();

    assert!(!binding.is_current_for(&world));
}

#[test]
fn replacement_world_advances_scene_binding_generation_past_reused_entity_ids() {
    let mut current = World::empty();
    let current_root = current.spawn_node(NodeKind::Empty);
    let current_hero = current.spawn_node(NodeKind::Mesh);
    current.rename_node(current_root, "Root").unwrap();
    current.rename_node(current_hero, "Hero").unwrap();
    current
        .set_parent_checked(current_hero, Some(current_root))
        .unwrap();
    let retired_writer = current
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation").unwrap(),
        )
        .unwrap()
        .unwrap();

    let mut replacement = World::empty();
    let replacement_root = replacement.spawn_node(NodeKind::Empty);
    let replacement_hero = replacement.spawn_node(NodeKind::Mesh);
    replacement.rename_node(replacement_root, "Root").unwrap();
    replacement.rename_node(replacement_hero, "Hero").unwrap();
    replacement
        .set_parent_checked(replacement_hero, Some(replacement_root))
        .unwrap();
    assert_eq!(replacement_root, current_root);
    assert_eq!(replacement_hero, current_hero);

    replacement.advance_scene_binding_generations_after(&current);

    assert!(
        replacement.scene_binding_generation(replacement_root)
            > current.scene_binding_generation(current_root)
    );
    assert!(!retired_writer.is_current_for(&replacement));
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
fn compiled_scene_property_writer_reads_without_path_dispatch_and_requires_rebind_after_name_change(
) {
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
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation").unwrap(),
        )
        .unwrap();
    let accessor = accessor.unwrap();

    assert_eq!(
        world.read_compiled_scene_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Vec3([
            1.0, 2.0, 3.0,
        ]))
    );

    world
        .update_transform(hero, Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)))
        .unwrap();
    assert_eq!(
        world.read_compiled_scene_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Vec3([
            4.0, 5.0, 6.0,
        ]))
    );

    world.rename_node(hero, "Renamed hero").unwrap();
    assert_eq!(world.read_compiled_scene_property(&accessor), None);
}

#[test]
fn compiled_scene_property_writer_writes_without_string_dispatch() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    let accessor = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
        )
        .unwrap();
    let accessor = accessor.unwrap();

    assert!(world
        .write_compiled_scene_property(
            &accessor,
            crate::core::framework::scene::ScenePropertyValue::Scalar(7.0),
        )
        .unwrap());
    assert_eq!(
        world.read_compiled_scene_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            7.0
        ))
    );
    assert!(!world
        .write_compiled_scene_property(
            &accessor,
            crate::core::framework::scene::ScenePropertyValue::Scalar(7.0),
        )
        .unwrap());
}

#[test]
fn compiled_scene_property_writer_writes_mesh_morph_weights_without_path_dispatch() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    let accessor = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("MeshRenderer.morph_weights.2").unwrap(),
        )
        .unwrap()
        .unwrap();

    assert!(world
        .write_compiled_scene_property(
            &accessor,
            crate::core::framework::scene::ScenePropertyValue::Scalar(0.75),
        )
        .unwrap());
    assert_eq!(
        world
            .get::<MeshRenderer>(hero)
            .unwrap()
            .morph_weights
            .as_slice(),
        &[0.0, 0.0, 0.75]
    );
    assert_eq!(
        world.read_compiled_scene_property(&accessor),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            0.75
        ))
    );
    assert!(!world
        .write_compiled_scene_property(
            &accessor,
            crate::core::framework::scene::ScenePropertyValue::Scalar(0.75),
        )
        .unwrap());
}

#[test]
fn compiled_scene_property_writer_writes_camera_and_light_aliases_without_path_dispatch() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let camera = world.spawn_node(NodeKind::Camera);
    let light = world.spawn_node(NodeKind::DirectionalLight);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(camera, "Camera").unwrap();
    world.rename_node(light, "Key light").unwrap();
    world.set_parent_checked(camera, Some(root)).unwrap();
    world.set_parent_checked(light, Some(root)).unwrap();

    let camera_fov = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Camera").unwrap(),
            &ComponentPropertyPath::parse("Camera.fov_y_radians").unwrap(),
        )
        .unwrap()
        .unwrap();
    let light_intensity = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Key light").unwrap(),
            &ComponentPropertyPath::parse("Light.intensity").unwrap(),
        )
        .unwrap()
        .unwrap();

    let generation_before_camera_write = world.world_generation();
    assert!(world
        .write_compiled_scene_property(
            &camera_fov,
            crate::core::framework::scene::ScenePropertyValue::Scalar(1.2),
        )
        .unwrap());
    assert_eq!(world.world_generation(), generation_before_camera_write + 1);
    let generation_before_light_write = world.world_generation();
    assert!(world
        .write_compiled_scene_property(
            &light_intensity,
            crate::core::framework::scene::ScenePropertyValue::Scalar(2.5),
        )
        .unwrap());
    assert_eq!(world.world_generation(), generation_before_light_write + 1);
    assert_eq!(
        world.read_compiled_scene_property(&camera_fov),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            1.2
        ))
    );
    assert_eq!(
        world.read_compiled_scene_property(&light_intensity),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            2.5
        ))
    );
    assert!(!world
        .write_compiled_scene_property(
            &light_intensity,
            crate::core::framework::scene::ScenePropertyValue::Scalar(2.5),
        )
        .unwrap());
    assert_eq!(world.world_generation(), generation_before_light_write + 1);
}

#[test]
fn compiled_scene_property_writer_writes_dynamic_fields_and_ignores_unrelated_schema_changes() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .register_component_type(
            ComponentTypeDescriptor::new("weather.Component.CloudLayer", "weather", "Cloud Layer")
                .with_property("coverage", "Scalar", true),
        )
        .unwrap();
    world
        .set_dynamic_component(
            hero,
            "weather.Component.CloudLayer",
            json!({ "coverage": 0.5 }),
        )
        .unwrap();

    let exact_schema_key = world
        .compile_scene_property_target(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.coverage").unwrap(),
        )
        .unwrap();
    let case_distinct_schema_key = world
        .compile_scene_property_target(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("weather.component.cloudlayer.coverage").unwrap(),
        )
        .unwrap();
    assert_ne!(
        exact_schema_key.component_field_id(),
        case_distinct_schema_key.component_field_id()
    );

    let writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("weather.Component.CloudLayer.coverage").unwrap(),
        )
        .unwrap()
        .unwrap();
    let cloud_schema_generation =
        world.component_type_schema_generation("weather.Component.CloudLayer");
    assert_ne!(cloud_schema_generation, 0);
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            0.5
        ))
    );

    let generation_before_write = world.world_generation();
    assert!(world
        .write_compiled_scene_property(
            &writer,
            crate::core::framework::scene::ScenePropertyValue::Scalar(0.75),
        )
        .unwrap());
    assert_eq!(world.world_generation(), generation_before_write + 1);
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            0.75
        ))
    );

    world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.Component.Wind",
            "weather",
            "Wind",
        ))
        .unwrap();
    assert_eq!(
        world.component_type_schema_generation("weather.Component.CloudLayer"),
        cloud_schema_generation
    );
    assert!(writer.is_current_for(&world));
}

#[test]
fn compiled_undeclared_dynamic_writer_stales_when_schema_catalog_becomes_restrictive() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    let writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("weather.Component.Legacy.coverage").unwrap(),
        )
        .unwrap()
        .unwrap();
    assert!(writer.is_current_for(&world));

    world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap();
    assert!(!writer.is_current_for(&world));
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

    world.remove_entity(root).unwrap();
    world.insert_node_record(root_record).unwrap();

    assert!(!target.is_current_for(&world));
}

#[test]
fn compiled_scene_property_writer_stable_access_avoids_text_and_entry_work() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();

    world.reset_compiled_scene_property_access_stats();
    let writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
        )
        .unwrap()
        .unwrap();
    let compile_stats = world.compiled_scene_property_access_stats();
    assert_eq!(compile_stats.path_lookup_requests, 1);
    assert!(compile_stats.path_entity_visits >= 1);
    assert!(compile_stats.path_ancestor_visits >= 1);
    assert!(compile_stats.path_sibling_visits >= 1);
    assert_eq!(compile_stats.target_compilations, 1);
    assert_eq!(compile_stats.field_dispatch_compilations, 1);
    assert_eq!(
        compile_stats.canonicalization_bytes,
        "transform.translation.x".len() as u64
    );
    assert_eq!(compile_stats.property_entry_visits, 0);

    let stable_baseline = world.compiled_scene_property_access_stats();
    assert!(world
        .write_compiled_scene_property(
            &writer,
            crate::core::framework::scene::ScenePropertyValue::Scalar(5.0),
        )
        .unwrap());
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(crate::core::framework::scene::ScenePropertyValue::Scalar(
            5.0
        ))
    );
    let stable_stats = world
        .compiled_scene_property_access_stats()
        .saturating_delta_since(stable_baseline);

    assert_eq!(stable_stats.compiled_writer_accesses, 1);
    assert_eq!(stable_stats.compiled_reader_accesses, 1);
    assert_eq!(stable_stats.path_lookup_requests, 0);
    assert_eq!(stable_stats.path_entity_visits, 0);
    assert_eq!(stable_stats.path_ancestor_visits, 0);
    assert_eq!(stable_stats.path_sibling_visits, 0);
    assert_eq!(stable_stats.canonicalization_bytes, 0);
    assert_eq!(stable_stats.target_compilations, 0);
    assert_eq!(stable_stats.field_dispatch_compilations, 0);
    assert_eq!(stable_stats.property_entry_visits, 0);

    let stale_baseline = world.compiled_scene_property_access_stats();
    world.rename_node(hero, "Renamed Hero").unwrap();
    assert_eq!(world.read_compiled_scene_property(&writer), None);
    let stale_stats = world
        .compiled_scene_property_access_stats()
        .saturating_delta_since(stale_baseline);
    assert_eq!(stale_stats.stale_target_rejections, 1);
    assert_eq!(stale_stats.compiled_reader_accesses, 0);
}

#[test]
fn compiled_scene_property_writer_scales_stable_access_without_text_work() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(root, "Root").unwrap();
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    let writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
        )
        .unwrap()
        .unwrap();

    let mut next_value = 1_u64;
    for track_count in [1_u64, 100, 10_000] {
        world.reset_compiled_scene_property_access_stats();
        for _ in 0..track_count {
            assert!(world
                .write_compiled_scene_property(
                    &writer,
                    crate::core::framework::scene::ScenePropertyValue::Scalar(next_value as _),
                )
                .unwrap());
            next_value = next_value.saturating_add(1);
        }

        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.compiled_writer_accesses, track_count);
        assert_eq!(stats.path_lookup_requests, 0);
        assert_eq!(stats.path_entity_visits, 0);
        assert_eq!(stats.path_ancestor_visits, 0);
        assert_eq!(stats.path_sibling_visits, 0);
        assert_eq!(stats.canonicalization_bytes, 0);
        assert_eq!(stats.field_dispatch_compilations, 0);
        assert_eq!(stats.property_entry_visits, 0);
    }
}

#[test]
fn compiled_scene_property_access_diagnostics_reset_for_clone_and_deserialize() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    world.rename_node(root, "Root").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let entity_path = EntityPath::parse("Root").unwrap();
    world
        .compile_scene_property_writer(&entity_path, &property_path)
        .unwrap()
        .unwrap();
    assert_ne!(
        world.compiled_scene_property_access_stats(),
        crate::scene::world::CompiledScenePropertyAccessStats::default()
    );

    let cloned = world.clone();
    assert_eq!(
        cloned.compiled_scene_property_access_stats(),
        crate::scene::world::CompiledScenePropertyAccessStats::default()
    );

    let decoded: World = serde_json::from_value(serde_json::to_value(&world).unwrap()).unwrap();
    assert_eq!(
        decoded.compiled_scene_property_access_stats(),
        crate::scene::world::CompiledScenePropertyAccessStats::default()
    );
}
