use crate::core::framework::scene::{ComponentPropertyPath, ComponentTypeDescriptor, EntityPath};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::MeshRenderer;
use crate::scene::{NodeKind, World};
use serde_json::json;

#[test]
fn compiled_descendant_name_index_omits_root_and_preserves_hierarchy_order() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let arm = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let hand = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let sibling = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let unrelated_root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let unrelated_child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let alternate_root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
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
    let spawned = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(spawned, Some(root)).unwrap();
    assert!(!before_spawn.is_current_for(&world));

    let before_remove = world.compile_descendant_name_index(root).unwrap();
    world.remove_entity(spawned).unwrap();
    assert!(!before_remove.is_current_for(&world));
}

#[test]
fn compiled_descendant_name_index_stales_when_root_id_is_reused() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let current_root = current
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let current_hero = current
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let replacement_root = replacement
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let replacement_hero = replacement
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let alternate_root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(child, Some(root)).unwrap();
    let root_binding = world.compile_descendant_name_index(root).unwrap();
    let alternate_binding = world.compile_descendant_name_index(alternate_root).unwrap();

    assert!(world.corrupt_hierarchy_parent_for_tests(child, Some(alternate_root)));

    assert!(!root_binding.is_current_for(&world));
    assert!(!alternate_binding.is_current_for(&world));
}

#[test]
fn compiled_scene_property_target_interns_ids_and_stales_for_its_path_changes() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let unrelated_root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let unrelated_child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
fn compiled_scene_property_writer_rejects_morph_weight_index_overflow() {
    let mut world = World::empty();
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.rename_node(hero, "Hero").unwrap();
    let accessor = world
        .compile_scene_property_writer(
            &EntityPath::parse("Hero").unwrap(),
            &ComponentPropertyPath::parse(&format!("MeshRenderer.morph_weights.{}", usize::MAX))
                .unwrap(),
        )
        .unwrap()
        .unwrap();

    let error = world
        .write_compiled_scene_property(
            &accessor,
            crate::core::framework::scene::ScenePropertyValue::Scalar(0.75),
        )
        .expect_err("usize::MAX cannot be converted to a morph-weight vector length");
    assert!(matches!(
        error,
        crate::scene::SceneError::InvalidPropertyIndex { .. }
    ));
    assert!(world
        .get::<MeshRenderer>(hero)
        .unwrap()
        .morph_weights
        .is_empty());
}

#[test]
fn compiled_scene_property_writer_writes_camera_and_light_aliases_without_path_dispatch() {
    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let camera = world
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");
    let light = world
        .spawn_node(NodeKind::DirectionalLight)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
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
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
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

#[test]
fn compiled_light_writes_change_and_repeat_all_fields_without_false_generations() {
    use crate::core::framework::scene::ScenePropertyValue;

    let cases = [
        (
            NodeKind::AmbientLight,
            "AmbientLight.color",
            ScenePropertyValue::Vec3([0.2, 0.4, 0.6]),
        ),
        (
            NodeKind::AmbientLight,
            "AmbientLight.intensity",
            ScenePropertyValue::Scalar(2.5),
        ),
        (
            NodeKind::AmbientLight,
            "AmbientLight.affects_lightmapped_meshes",
            ScenePropertyValue::Bool(false),
        ),
        (
            NodeKind::DirectionalLight,
            "DirectionalLight.direction",
            ScenePropertyValue::Vec3([0.25, -0.5, 0.75]),
        ),
        (
            NodeKind::DirectionalLight,
            "DirectionalLight.color",
            ScenePropertyValue::Vec3([0.2, 0.4, 0.6]),
        ),
        (
            NodeKind::DirectionalLight,
            "DirectionalLight.intensity",
            ScenePropertyValue::Scalar(2.5),
        ),
        (
            NodeKind::PointLight,
            "PointLight.color",
            ScenePropertyValue::Vec3([0.2, 0.4, 0.6]),
        ),
        (
            NodeKind::PointLight,
            "PointLight.intensity",
            ScenePropertyValue::Scalar(2.5),
        ),
        (
            NodeKind::PointLight,
            "PointLight.range",
            ScenePropertyValue::Scalar(23.0),
        ),
        (
            NodeKind::RectLight,
            "RectLight.color",
            ScenePropertyValue::Vec3([0.2, 0.4, 0.6]),
        ),
        (
            NodeKind::RectLight,
            "RectLight.intensity",
            ScenePropertyValue::Scalar(2.5),
        ),
        (
            NodeKind::RectLight,
            "RectLight.range",
            ScenePropertyValue::Scalar(23.0),
        ),
        (
            NodeKind::RectLight,
            "RectLight.size",
            ScenePropertyValue::Vec2([2.0, 3.0]),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.direction",
            ScenePropertyValue::Vec3([0.25, -0.5, 0.75]),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.color",
            ScenePropertyValue::Vec3([0.2, 0.4, 0.6]),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.intensity",
            ScenePropertyValue::Scalar(2.5),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.range",
            ScenePropertyValue::Scalar(23.0),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.inner_angle_radians",
            ScenePropertyValue::Scalar(0.2),
        ),
        (
            NodeKind::SpotLight,
            "SpotLight.outer_angle_radians",
            ScenePropertyValue::Scalar(0.8),
        ),
    ];

    for (kind, path, value) in cases {
        let mut world = World::empty();
        let light = world.spawn_node(kind).unwrap();
        world.rename_node(light, "Light").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Light").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        assert_ne!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let cached_before = world.node_records();
        let generation = world.world_generation();
        world.reset_compiled_scene_property_access_stats();

        assert!(
            world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let cached_after = world.node_records();
        assert_ne!(cached_after, cached_before, "{path}");
        assert!(writer.is_current_for(&world), "{path}");
        assert!(
            !world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(world.node_records(), cached_after, "{path}");

        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.compiled_writer_accesses, 2, "{path}");
        assert_eq!(stats.compiled_reader_accesses, 1, "{path}");
        assert_eq!(stats.path_lookup_requests, 0, "{path}");
        assert_eq!(stats.canonicalization_bytes, 0, "{path}");
        assert_eq!(stats.target_compilations, 0, "{path}");
        assert_eq!(stats.field_dispatch_compilations, 0, "{path}");
        assert_eq!(stats.property_entry_visits, 0, "{path}");

        world.rename_node(light, "Renamed light").unwrap();
        let stale_generation = world.world_generation();
        world.reset_compiled_scene_property_access_stats();
        for value in [value, ScenePropertyValue::Scalar(f32::NAN)] {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            assert!(
                matches!(error, crate::scene::SceneError::PropertyUnavailable { entity, .. } if entity == light),
                "{path}"
            );
            assert_eq!(world.world_generation(), stale_generation, "{path}");
        }
        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.stale_target_rejections, 2, "{path}");
        assert_eq!(stats.compiled_writer_accesses, 0, "{path}");
    }
}

#[test]
fn compiled_light_writes_reject_invalid_values_before_mutation() {
    use crate::core::framework::scene::ScenePropertyValue;

    let cases = [
        (
            NodeKind::DirectionalLight,
            "DirectionalLight.intensity",
            vec![
                (ScenePropertyValue::Bool(true), false),
                (ScenePropertyValue::Scalar(f32::NAN), true),
                (ScenePropertyValue::Scalar(f32::INFINITY), true),
                (ScenePropertyValue::Scalar(f32::NEG_INFINITY), true),
            ],
        ),
        (
            NodeKind::DirectionalLight,
            "DirectionalLight.color",
            vec![
                (ScenePropertyValue::Vec2([1.0, 2.0]), false),
                (ScenePropertyValue::Vec3([f32::NAN, 1.0, 2.0]), true),
                (ScenePropertyValue::Vec3([1.0, f32::INFINITY, 2.0]), true),
                (
                    ScenePropertyValue::Vec3([1.0, 2.0, f32::NEG_INFINITY]),
                    true,
                ),
            ],
        ),
        (
            NodeKind::RectLight,
            "RectLight.size",
            vec![
                (ScenePropertyValue::Vec3([1.0, 2.0, 3.0]), false),
                (ScenePropertyValue::Vec2([f32::NAN, 1.0]), true),
                (ScenePropertyValue::Vec2([1.0, f32::INFINITY]), true),
            ],
        ),
        (
            NodeKind::AmbientLight,
            "AmbientLight.affects_lightmapped_meshes",
            vec![
                (ScenePropertyValue::Scalar(1.0), false),
                (ScenePropertyValue::String("false".to_string()), false),
            ],
        ),
    ];

    for (kind, path, invalid_values) in cases {
        let mut world = World::empty();
        let light = world.spawn_node(kind).unwrap();
        world.rename_node(light, "Light").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Light").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let value_before = world.read_compiled_scene_property(&writer);
        let cached_before = world.node_records();
        let generation = world.world_generation();

        for (value, nonfinite) in invalid_values {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            if nonfinite {
                assert!(
                    matches!(
                        error,
                        crate::scene::SceneError::NonFinitePropertyValue { .. }
                    ),
                    "{path}"
                );
            } else {
                assert!(
                    matches!(error, crate::scene::SceneError::PropertyTypeMismatch { .. }),
                    "{path}"
                );
            }
            assert_eq!(world.world_generation(), generation, "{path}");
            assert_eq!(
                world.read_compiled_scene_property(&writer),
                value_before,
                "{path}"
            );
            assert_eq!(world.node_records(), cached_before, "{path}");
        }
    }
}

#[test]
fn compiled_light_writes_preserve_scalar_coercion_and_exact_float_equality() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::components::DirectionalLight;

    let mut world = World::empty();
    let light = world.spawn_node(NodeKind::DirectionalLight).unwrap();
    world.rename_node(light, "Light").unwrap();
    let writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Light").unwrap(),
            &ComponentPropertyPath::parse("Light.intensity").unwrap(),
        )
        .unwrap()
        .unwrap();
    let generation = world.world_generation();
    assert!(world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Integer(7))
        .unwrap());
    assert_eq!(world.world_generation(), generation + 1);
    for value in [
        ScenePropertyValue::Unsigned(7),
        ScenePropertyValue::Scalar(7.0),
    ] {
        assert!(!world.write_compiled_scene_property(&writer, value).unwrap());
        assert_eq!(world.world_generation(), generation + 1);
    }
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(ScenePropertyValue::Scalar(7.0))
    );

    assert!(world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    let zero_generation = world.world_generation();
    assert!(!world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(-0.0))
        .unwrap());
    assert_eq!(world.world_generation(), zero_generation);
    assert!(world
        .get::<DirectionalLight>(light)
        .unwrap()
        .intensity
        .is_sign_positive());

    assert!(world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(f32::from_bits(1)))
        .unwrap());
    assert_eq!(world.world_generation(), zero_generation + 1);
    world.get_mut::<DirectionalLight>(light).unwrap().intensity = f32::NAN;
    let nan_generation = world.world_generation();
    assert!(world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(7.0))
        .unwrap());
    assert_eq!(world.world_generation(), nan_generation + 1);
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(ScenePropertyValue::Scalar(7.0))
    );
}

#[test]
fn compiled_camera_mesh_writes_change_and_repeat_all_fields_without_false_generations() {
    use crate::core::framework::scene::ScenePropertyValue;

    let cases = [
        (
            NodeKind::Camera,
            "Camera.fov_y_radians",
            ScenePropertyValue::Scalar(1.25),
        ),
        (
            NodeKind::Camera,
            "Camera.z_near",
            ScenePropertyValue::Scalar(0.125),
        ),
        (
            NodeKind::Camera,
            "Camera.z_far",
            ScenePropertyValue::Scalar(2048.0),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.render_queue",
            ScenePropertyValue::Integer(17),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.material_queue",
            ScenePropertyValue::Integer(23),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.order_in_layer",
            ScenePropertyValue::Integer(31),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.depth_bias",
            ScenePropertyValue::Scalar(0.25),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.morph_weights.2",
            ScenePropertyValue::Scalar(0.75),
        ),
        (
            NodeKind::Mesh,
            "MeshRenderer.tint",
            ScenePropertyValue::Vec4([0.2, 0.4, 0.6, 0.8]),
        ),
    ];

    for (kind, path, value) in cases {
        let mut world = World::empty();
        let entity = world.spawn_node(kind).unwrap();
        world.rename_node(entity, "Target").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        assert_ne!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let generation = world.world_generation();
        world.reset_compiled_scene_property_access_stats();

        assert!(
            world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let projection_after = world.node_records();
        assert!(writer.is_current_for(&world), "{path}");
        assert!(
            !world.write_compiled_scene_property(&writer, value).unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(world.node_records(), projection_after, "{path}");

        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.compiled_writer_accesses, 2, "{path}");
        assert_eq!(stats.compiled_reader_accesses, 1, "{path}");
        assert_eq!(stats.path_lookup_requests, 0, "{path}");
        assert_eq!(stats.path_entity_visits, 0, "{path}");
        assert_eq!(stats.path_ancestor_visits, 0, "{path}");
        assert_eq!(stats.path_sibling_visits, 0, "{path}");
        assert_eq!(stats.canonicalization_bytes, 0, "{path}");
        assert_eq!(stats.target_compilations, 0, "{path}");
        assert_eq!(stats.field_dispatch_compilations, 0, "{path}");
        assert_eq!(stats.property_entry_visits, 0, "{path}");
        assert_eq!(stats.stale_target_rejections, 0, "{path}");
    }
}

#[test]
fn compiled_camera_mesh_writes_reject_invalid_values_before_mutation() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;

    let mut cases = Vec::new();
    for (kind, path) in [
        (NodeKind::Camera, "Camera.fov_y_radians"),
        (NodeKind::Camera, "Camera.z_near"),
        (NodeKind::Camera, "Camera.z_far"),
        (NodeKind::Mesh, "MeshRenderer.depth_bias"),
        (NodeKind::Mesh, "MeshRenderer.morph_weights.2"),
    ] {
        cases.push((
            kind,
            path,
            vec![
                (ScenePropertyValue::Bool(true), Some("scalar")),
                (ScenePropertyValue::String("7".to_string()), Some("scalar")),
                (ScenePropertyValue::Scalar(f32::NAN), None),
                (ScenePropertyValue::Scalar(f32::INFINITY), None),
                (ScenePropertyValue::Scalar(f32::NEG_INFINITY), None),
            ],
        ));
    }
    for path in [
        "MeshRenderer.render_queue",
        "MeshRenderer.material_queue",
        "MeshRenderer.order_in_layer",
    ] {
        cases.push((
            NodeKind::Mesh,
            path,
            vec![
                (ScenePropertyValue::Scalar(7.0), Some("integer")),
                (ScenePropertyValue::Scalar(f32::NAN), Some("integer")),
                (ScenePropertyValue::Bool(true), Some("integer")),
                (ScenePropertyValue::Integer(i64::MAX), Some("i32 integer")),
                (ScenePropertyValue::Integer(i64::MIN), Some("i32 integer")),
                (ScenePropertyValue::Unsigned(u64::MAX), Some("i32 integer")),
            ],
        ));
    }
    cases.push((
        NodeKind::Mesh,
        "MeshRenderer.tint",
        vec![
            (ScenePropertyValue::Vec3([1.0, 2.0, 3.0]), Some("vec4")),
            (ScenePropertyValue::Vec4([f32::NAN, 1.0, 2.0, 3.0]), None),
            (
                ScenePropertyValue::Vec4([1.0, f32::INFINITY, 2.0, 3.0]),
                None,
            ),
            (
                ScenePropertyValue::Vec4([1.0, 2.0, f32::NEG_INFINITY, 3.0]),
                None,
            ),
            (ScenePropertyValue::Vec4([1.0, 2.0, 3.0, f32::NAN]), None),
        ],
    ));

    for (kind, path, invalid_values) in cases {
        let mut world = World::empty();
        let entity = world.spawn_node(kind).unwrap();
        world.rename_node(entity, "Target").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let value_before = world.read_compiled_scene_property(&writer);
        let projection_before = world.node_records();
        let generation = world.world_generation();

        for (value, expected_type) in invalid_values {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            if let Some(expected_type) = expected_type {
                assert!(
                    matches!(error, SceneError::PropertyTypeMismatch { ref property_path, ref expected }
                    if property_path == path && expected == expected_type),
                    "{path}"
                );
            } else {
                assert!(
                    matches!(error, SceneError::NonFinitePropertyValue { ref property_path, .. }
                    if property_path == path),
                    "{path}"
                );
            }
            assert_eq!(world.world_generation(), generation, "{path}");
            assert_eq!(
                world.read_compiled_scene_property(&writer),
                value_before,
                "{path}"
            );
            assert_eq!(world.node_records(), projection_before, "{path}");
            assert!(writer.is_current_for(&world), "{path}");
        }
    }
}

#[test]
fn compiled_camera_mesh_writes_prioritize_missing_components_and_stale_targets() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;

    let cases = [
        (
            "Camera.fov_y_radians".to_string(),
            "write compiled camera property",
            "Camera",
            ScenePropertyValue::Scalar(f32::NAN),
        ),
        (
            "Camera.z_near".to_string(),
            "write compiled camera property",
            "Camera",
            ScenePropertyValue::Scalar(f32::INFINITY),
        ),
        (
            "Camera.z_far".to_string(),
            "write compiled camera property",
            "Camera",
            ScenePropertyValue::Scalar(f32::NEG_INFINITY),
        ),
        (
            "MeshRenderer.render_queue".to_string(),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Integer(i64::MAX),
        ),
        (
            "MeshRenderer.material_queue".to_string(),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Unsigned(u64::MAX),
        ),
        (
            "MeshRenderer.order_in_layer".to_string(),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Scalar(f32::NAN),
        ),
        (
            "MeshRenderer.depth_bias".to_string(),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Scalar(f32::NAN),
        ),
        (
            format!("MeshRenderer.morph_weights.{}", usize::MAX),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Scalar(f32::NAN),
        ),
        (
            "MeshRenderer.tint".to_string(),
            "write compiled mesh renderer property",
            "MeshRenderer",
            ScenePropertyValue::Vec4([f32::NAN, 0.0, 0.0, 0.0]),
        ),
    ];

    for (path, operation, component, invalid_value) in cases {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world.rename_node(entity, "Target").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(&path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let generation = world.world_generation();
        let projection_before = world.node_records();
        assert_eq!(world.read_compiled_scene_property(&writer), None);

        for value in [ScenePropertyValue::Bool(true), invalid_value] {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            assert!(
                matches!(error, SceneError::MissingRequiredComponent {
                operation: actual_operation, entity: actual_entity, component: actual_component,
            } if actual_operation == operation && actual_entity == entity && actual_component == component),
                "{path}"
            );
            assert_eq!(world.world_generation(), generation, "{path}");
            assert_eq!(world.node_records(), projection_before, "{path}");
            assert!(writer.is_current_for(&world), "{path}");
        }

        world.rename_node(entity, "Renamed target").unwrap();
        let stale_generation = world.world_generation();
        let stale_projection = world.node_records();
        world.reset_compiled_scene_property_access_stats();
        for value in [
            ScenePropertyValue::Bool(true),
            ScenePropertyValue::Scalar(f32::NAN),
        ] {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            assert!(
                matches!(error, SceneError::PropertyUnavailable { entity: actual_entity, ref property_path }
                if actual_entity == entity && property_path == &path),
                "{path}"
            );
            assert_eq!(world.world_generation(), stale_generation, "{path}");
            assert_eq!(world.node_records(), stale_projection, "{path}");
        }
        assert_eq!(world.read_compiled_scene_property(&writer), None);
        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.stale_target_rejections, 3, "{path}");
        assert_eq!(stats.compiled_writer_accesses, 0, "{path}");
        assert_eq!(stats.compiled_reader_accesses, 0, "{path}");
    }
}

#[test]
fn compiled_camera_mesh_writes_preserve_numeric_coercion_and_exact_float_equality() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::core::math::Vec4;
    use crate::scene::components::CameraComponent;

    for (kind, path) in [
        (NodeKind::Camera, "Camera.fov_y_radians"),
        (NodeKind::Camera, "Camera.z_near"),
        (NodeKind::Camera, "Camera.z_far"),
        (NodeKind::Mesh, "MeshRenderer.depth_bias"),
        (NodeKind::Mesh, "MeshRenderer.morph_weights.0"),
    ] {
        let mut world = World::empty();
        let entity = world.spawn_node(kind).unwrap();
        world.rename_node(entity, "Target").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let generation = world.world_generation();
        assert!(
            world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Integer(16_777_217))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        for value in [
            ScenePropertyValue::Unsigned(16_777_217),
            ScenePropertyValue::Scalar(16_777_216.0),
        ] {
            assert!(
                !world.write_compiled_scene_property(&writer, value).unwrap(),
                "{path}"
            );
            assert_eq!(world.world_generation(), generation + 1, "{path}");
        }
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Scalar(16_777_216.0)),
            "{path}"
        );

        assert!(
            world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.0))
                .unwrap(),
            "{path}"
        );
        let zero_generation = world.world_generation();
        assert!(
            !world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(-0.0))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), zero_generation, "{path}");
        let Some(ScenePropertyValue::Scalar(stored_zero)) =
            world.read_compiled_scene_property(&writer)
        else {
            panic!("scalar read expected for {path}");
        };
        assert!(stored_zero.is_sign_positive(), "{path}");
        assert!(
            world
                .write_compiled_scene_property(
                    &writer,
                    ScenePropertyValue::Scalar(f32::from_bits(1))
                )
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), zero_generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Scalar(f32::from_bits(1))),
            "{path}"
        );
    }

    for path in [
        "MeshRenderer.render_queue",
        "MeshRenderer.material_queue",
        "MeshRenderer.order_in_layer",
    ] {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Mesh).unwrap();
        world.rename_node(entity, "Mesh").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Mesh").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let generation = world.world_generation();
        assert!(
            world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Integer(7))
                .unwrap(),
            "{path}"
        );
        assert!(
            !world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Unsigned(7))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Integer(7)),
            "{path}"
        );
        assert!(
            world
                .write_compiled_scene_property(
                    &writer,
                    ScenePropertyValue::Integer(i64::from(i32::MIN))
                )
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 2, "{path}");
        assert!(
            world
                .write_compiled_scene_property(
                    &writer,
                    ScenePropertyValue::Unsigned(i32::MAX as u64)
                )
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 3, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Integer(i64::from(i32::MAX))),
            "{path}"
        );
    }

    let mut world = World::empty();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    let mesh = world.spawn_node(NodeKind::Mesh).unwrap();
    world.rename_node(camera, "Camera").unwrap();
    world.rename_node(mesh, "Mesh").unwrap();
    let tint_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse("MeshRenderer.tint").unwrap(),
        )
        .unwrap()
        .unwrap();
    assert!(world
        .write_compiled_scene_property(&tint_writer, ScenePropertyValue::Vec4([0.0; 4]))
        .unwrap());
    let tint_zero_generation = world.world_generation();
    assert!(!world
        .write_compiled_scene_property(&tint_writer, ScenePropertyValue::Vec4([-0.0; 4]))
        .unwrap());
    assert_eq!(world.world_generation(), tint_zero_generation);
    let Some(ScenePropertyValue::Vec4(stored_zero)) =
        world.read_compiled_scene_property(&tint_writer)
    else {
        panic!("tint read expected");
    };
    assert!(stored_zero.into_iter().all(f32::is_sign_positive));
    assert!(world
        .write_compiled_scene_property(
            &tint_writer,
            ScenePropertyValue::Vec4([f32::from_bits(1), 0.0, 0.0, 0.0])
        )
        .unwrap());
    assert_eq!(world.world_generation(), tint_zero_generation + 1);

    world
        .get_mut::<CameraComponent>(camera)
        .unwrap()
        .fov_y_radians = f32::NAN;
    let mesh_component = world.get_mut::<MeshRenderer>(mesh).unwrap();
    mesh_component.depth_bias = f32::NAN;
    mesh_component.morph_weights = vec![f32::NAN];
    mesh_component.tint = Vec4::splat(f32::NAN);
    for (entity_path, path, value) in [
        (
            "Camera",
            "Camera.fov_y_radians",
            ScenePropertyValue::Scalar(7.0),
        ),
        (
            "Mesh",
            "MeshRenderer.depth_bias",
            ScenePropertyValue::Scalar(7.0),
        ),
        (
            "Mesh",
            "MeshRenderer.morph_weights.0",
            ScenePropertyValue::Scalar(7.0),
        ),
        (
            "Mesh",
            "MeshRenderer.tint",
            ScenePropertyValue::Vec4([0.2, 0.4, 0.6, 0.8]),
        ),
    ] {
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse(entity_path).unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let generation = world.world_generation();
        assert!(
            world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(value),
            "{path}"
        );
    }
}

#[test]
fn compiled_mesh_morph_writes_preserve_growth_and_index_error_contracts() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;

    let mut world = World::empty();
    let mesh = world.spawn_node(NodeKind::Mesh).unwrap();
    world.rename_node(mesh, "Mesh").unwrap();
    let zero_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse("MeshRenderer.morph_weights.2").unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(world.read_compiled_scene_property(&zero_writer), None);
    let generation = world.world_generation();
    assert!(world
        .write_compiled_scene_property(&zero_writer, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(
        world
            .get::<MeshRenderer>(mesh)
            .unwrap()
            .morph_weights
            .as_slice(),
        &[0.0, 0.0, 0.0]
    );
    assert_eq!(
        world.read_compiled_scene_property(&zero_writer),
        Some(ScenePropertyValue::Scalar(0.0))
    );
    let projection_after_growth = world.node_records();
    assert!(!world
        .write_compiled_scene_property(&zero_writer, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(world.node_records(), projection_after_growth);

    let first_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse("MeshRenderer.morph_weights.0").unwrap(),
        )
        .unwrap()
        .unwrap();
    assert!(world
        .write_compiled_scene_property(&first_writer, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    let gap_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse("MeshRenderer.morph_weights.5").unwrap(),
        )
        .unwrap()
        .unwrap();
    let gap_generation = world.world_generation();
    assert!(world
        .write_compiled_scene_property(&gap_writer, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert_eq!(world.world_generation(), gap_generation + 1);
    assert_eq!(
        world
            .get::<MeshRenderer>(mesh)
            .unwrap()
            .morph_weights
            .as_slice(),
        &[0.75, 0.0, 0.0, 0.0, 0.0, 0.0]
    );

    let overflow_path = format!("MeshRenderer.morph_weights.{}", usize::MAX);
    let overflow_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse(&overflow_path).unwrap(),
        )
        .unwrap()
        .unwrap();
    let overflow_generation = world.world_generation();
    let projection_before_errors = world.node_records();
    let error = world
        .write_compiled_scene_property(&overflow_writer, ScenePropertyValue::Scalar(0.0))
        .unwrap_err();
    assert!(
        matches!(error, SceneError::InvalidPropertyIndex { ref property_path, index_kind: "morph weight index" }
        if property_path == &overflow_path)
    );
    assert_eq!(world.world_generation(), overflow_generation);
    let error = world
        .write_compiled_scene_property(&overflow_writer, ScenePropertyValue::Bool(true))
        .unwrap_err();
    assert!(
        matches!(error, SceneError::PropertyTypeMismatch { ref property_path, ref expected }
        if property_path == &overflow_path && expected == "scalar")
    );
    let error = world
        .write_compiled_scene_property(&overflow_writer, ScenePropertyValue::Scalar(f32::NAN))
        .unwrap_err();
    assert!(
        matches!(error, SceneError::NonFinitePropertyValue { ref property_path, .. }
        if property_path == &overflow_path)
    );
    assert_eq!(world.world_generation(), overflow_generation);
    assert_eq!(world.node_records(), projection_before_errors);
    assert_eq!(world.read_compiled_scene_property(&overflow_writer), None);

    let reserve_path = format!("MeshRenderer.morph_weights.{}", usize::MAX - 1);
    let reserve_writer = world
        .compile_scene_property_writer(
            &EntityPath::parse("Mesh").unwrap(),
            &ComponentPropertyPath::parse(&reserve_path).unwrap(),
        )
        .unwrap()
        .unwrap();
    let reserve_generation = world.world_generation();
    let error = world
        .write_compiled_scene_property(&reserve_writer, ScenePropertyValue::Scalar(0.0))
        .unwrap_err();
    assert!(
        matches!(error, SceneError::InvalidPropertyIndex { ref property_path, index_kind: "morph weight index" }
        if property_path == &reserve_path)
    );
    // Capacity overflow reaches the in-place reserve and retains its mutation marker.
    assert_eq!(world.world_generation(), reserve_generation + 1);
    assert_eq!(world.node_records(), projection_before_errors);
    assert_eq!(world.read_compiled_scene_property(&reserve_writer), None);
}

fn compiled_animation_runtime_test_cases() -> [(
    &'static str,
    crate::core::framework::scene::ScenePropertyValue,
); 11] {
    use crate::core::framework::scene::ScenePropertyValue;

    [
        (
            "AnimationPlayer.playback_speed",
            ScenePropertyValue::Scalar(2.0),
        ),
        (
            "AnimationPlayer.time_seconds",
            ScenePropertyValue::Scalar(3.0),
        ),
        ("AnimationPlayer.weight", ScenePropertyValue::Scalar(0.75)),
        (
            "AnimationSequencePlayer.playback_speed",
            ScenePropertyValue::Scalar(4.0),
        ),
        (
            "AnimationSequencePlayer.time_seconds",
            ScenePropertyValue::Scalar(5.0),
        ),
        ("AnimationPlayer.looping", ScenePropertyValue::Bool(false)),
        ("AnimationPlayer.playing", ScenePropertyValue::Bool(false)),
        (
            "AnimationSequencePlayer.looping",
            ScenePropertyValue::Bool(false),
        ),
        (
            "AnimationSequencePlayer.playing",
            ScenePropertyValue::Bool(false),
        ),
        (
            "AnimationGraphPlayer.playing",
            ScenePropertyValue::Bool(false),
        ),
        (
            "AnimationStateMachinePlayer.playing",
            ScenePropertyValue::Bool(false),
        ),
    ]
}

fn compiled_animation_runtime_test_world() -> (World, crate::scene::EntityId) {
    use crate::core::resource::{ResourceHandle, ResourceId};
    use crate::scene::components::{
        AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
        AnimationStateMachinePlayerComponent,
    };

    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.rename_node(entity, "Target").unwrap();
    world
        .insert(
            entity,
            AnimationPlayerComponent {
                clip: ResourceHandle::new(ResourceId::from_stable_label("res://test/clip.zranim")),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            },
        )
        .unwrap();
    world
        .insert(
            entity,
            AnimationSequencePlayerComponent {
                sequence: ResourceHandle::new(ResourceId::from_stable_label(
                    "res://test/sequence.zranimseq",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                looping: true,
                playing: true,
            },
        )
        .unwrap();
    world
        .insert(
            entity,
            AnimationGraphPlayerComponent {
                graph: ResourceHandle::new(ResourceId::from_stable_label(
                    "res://test/graph.zranimgraph",
                )),
                parameters: Default::default(),
                playing: true,
            },
        )
        .unwrap();
    world
        .insert(
            entity,
            AnimationStateMachinePlayerComponent {
                state_machine: ResourceHandle::new(ResourceId::from_stable_label(
                    "res://test/state-machine.zranimsm",
                )),
                parameters: Default::default(),
                active_state: None,
                playing: true,
            },
        )
        .unwrap();
    (world, entity)
}

#[test]
fn compiled_animation_runtime_writes_change_and_repeat_all_fields_without_false_generations() {
    for (path, value) in compiled_animation_runtime_test_cases() {
        let (mut world, _) = compiled_animation_runtime_test_world();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        assert_ne!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let projection_before = world.node_records();
        let generation = world.world_generation();
        world.reset_compiled_scene_property_access_stats();

        assert!(
            world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        let projection_after = world.node_records();
        assert_ne!(projection_after, projection_before, "{path}");
        assert!(writer.is_current_for(&world), "{path}");
        assert!(
            !world.write_compiled_scene_property(&writer, value).unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(world.node_records(), projection_after, "{path}");

        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.compiled_writer_accesses, 2, "{path}");
        assert_eq!(stats.compiled_reader_accesses, 1, "{path}");
        assert_eq!(stats.path_lookup_requests, 0, "{path}");
        assert_eq!(stats.canonicalization_bytes, 0, "{path}");
        assert_eq!(stats.target_compilations, 0, "{path}");
        assert_eq!(stats.field_dispatch_compilations, 0, "{path}");
        assert_eq!(stats.property_entry_visits, 0, "{path}");
    }
}

#[test]
fn compiled_animation_runtime_writes_reject_invalid_values_before_mutation() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;

    for (path, valid_value) in compiled_animation_runtime_test_cases() {
        let invalid_values = if matches!(valid_value, ScenePropertyValue::Scalar(_)) {
            vec![
                (ScenePropertyValue::Bool(true), Some("scalar")),
                (ScenePropertyValue::String("7".to_string()), Some("scalar")),
                (ScenePropertyValue::Enum("7".to_string()), Some("scalar")),
                (ScenePropertyValue::Scalar(f32::NAN), None),
                (ScenePropertyValue::Scalar(f32::INFINITY), None),
                (ScenePropertyValue::Scalar(f32::NEG_INFINITY), None),
            ]
        } else {
            vec![
                (ScenePropertyValue::Scalar(0.0), Some("bool")),
                (ScenePropertyValue::Scalar(1.0), Some("bool")),
                (ScenePropertyValue::Scalar(f32::NAN), Some("bool")),
                (ScenePropertyValue::Integer(0), Some("bool")),
                (ScenePropertyValue::Integer(1), Some("bool")),
                (ScenePropertyValue::Unsigned(1), Some("bool")),
                (ScenePropertyValue::String("true".to_string()), Some("bool")),
                (ScenePropertyValue::Enum("true".to_string()), Some("bool")),
            ]
        };
        let (mut world, _) = compiled_animation_runtime_test_world();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let before = world.read_compiled_scene_property(&writer);
        let projection_before = world.node_records();
        let generation = world.world_generation();

        for (value, expected) in invalid_values {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            match expected {
                Some(expected) => assert!(
                    matches!(error, SceneError::PropertyTypeMismatch {
                        ref property_path, expected: ref actual,
                    } if property_path == path && actual == expected),
                    "{path}"
                ),
                None => assert!(
                    matches!(error, SceneError::NonFinitePropertyValue {
                        ref property_path, expected: "scalar",
                    } if property_path == path),
                    "{path}"
                ),
            }
            assert_eq!(world.world_generation(), generation, "{path}");
            assert_eq!(
                world.read_compiled_scene_property(&writer),
                before,
                "{path}"
            );
            assert_eq!(world.node_records(), projection_before, "{path}");
            assert!(writer.is_current_for(&world), "{path}");
        }
    }
}

#[test]
fn compiled_animation_runtime_writes_prioritize_missing_components_and_stale_targets() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;

    for (path, valid_value) in compiled_animation_runtime_test_cases() {
        let mut world = World::empty();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        world.rename_node(entity, "Target").unwrap();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let component = path.split('.').next().unwrap();
        let projection_before = world.node_records();
        let generation = world.world_generation();
        assert_eq!(world.read_compiled_scene_property(&writer), None);

        for value in [
            valid_value.clone(),
            ScenePropertyValue::Enum("invalid".to_string()),
            ScenePropertyValue::Scalar(f32::NAN),
        ] {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            assert!(
                matches!(error, SceneError::MissingRequiredComponent {
                    operation: "write compiled animation runtime property",
                    entity: actual_entity, component: actual_component,
                } if actual_entity == entity && actual_component == component),
                "{path}"
            );
            assert_eq!(world.world_generation(), generation, "{path}");
            assert_eq!(world.node_records(), projection_before, "{path}");
            assert!(writer.is_current_for(&world), "{path}");
        }

        world.rename_node(entity, "Renamed target").unwrap();
        let stale_generation = world.world_generation();
        let stale_projection = world.node_records();
        world.reset_compiled_scene_property_access_stats();
        for value in [
            valid_value,
            ScenePropertyValue::Enum("invalid".to_string()),
            ScenePropertyValue::Scalar(f32::NAN),
        ] {
            let error = world
                .write_compiled_scene_property(&writer, value)
                .unwrap_err();
            assert!(
                matches!(error, SceneError::PropertyUnavailable {
                    entity: actual_entity, ref property_path,
                } if actual_entity == entity && property_path == path),
                "{path}"
            );
            assert_eq!(world.world_generation(), stale_generation, "{path}");
            assert_eq!(world.node_records(), stale_projection, "{path}");
        }
        assert_eq!(world.read_compiled_scene_property(&writer), None);
        let stats = world.compiled_scene_property_access_stats();
        assert_eq!(stats.stale_target_rejections, 4, "{path}");
        assert_eq!(stats.compiled_writer_accesses, 0, "{path}");
        assert_eq!(stats.compiled_reader_accesses, 0, "{path}");
    }
}

#[test]
fn compiled_animation_runtime_writes_preserve_scalar_bool_coercion_and_exact_float_equality() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::components::{AnimationPlayerComponent, AnimationSequencePlayerComponent};

    for (path, value) in compiled_animation_runtime_test_cases() {
        if !matches!(value, ScenePropertyValue::Scalar(_)) {
            continue;
        }
        let (mut world, _) = compiled_animation_runtime_test_world();
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        let before = world.read_compiled_scene_property(&writer);
        let generation = world.world_generation();
        let changed = world
            .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.0))
            .unwrap();
        assert_eq!(
            changed,
            before != Some(ScenePropertyValue::Scalar(0.0)),
            "{path}"
        );
        assert_eq!(
            world.world_generation(),
            generation + u64::from(changed),
            "{path}"
        );
        let zero_generation = world.world_generation();
        assert!(
            !world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(-0.0))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), zero_generation, "{path}");
        let Some(ScenePropertyValue::Scalar(zero)) = world.read_compiled_scene_property(&writer)
        else {
            panic!("{path} must read as scalar");
        };
        assert_eq!(zero.to_bits(), 0.0f32.to_bits(), "{path}");

        for (input, expected) in [
            (ScenePropertyValue::Integer(-7), -7.0),
            (ScenePropertyValue::Unsigned(u64::MAX), u64::MAX as f32),
            (ScenePropertyValue::Integer(i64::MAX), i64::MAX as f32),
            (ScenePropertyValue::Integer(16_777_216), 16_777_216.0),
        ] {
            let generation = world.world_generation();
            assert!(
                world.write_compiled_scene_property(&writer, input).unwrap(),
                "{path}"
            );
            assert_eq!(world.world_generation(), generation + 1, "{path}");
            assert_eq!(
                world.read_compiled_scene_property(&writer),
                Some(ScenePropertyValue::Scalar(expected)),
                "{path}"
            );
        }
        let rounded_generation = world.world_generation();
        assert!(
            !world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Unsigned(16_777_217))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), rounded_generation, "{path}");

        let generation = world.world_generation();
        assert!(
            world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(1.0))
                .unwrap(),
            "{path}"
        );
        let adjacent = f32::from_bits(1.0f32.to_bits() + 1);
        assert!(
            world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(adjacent))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 2, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Scalar(adjacent)),
            "{path}"
        );
        assert!(
            !world
                .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(adjacent))
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 2, "{path}");
    }

    let (mut world, entity) = compiled_animation_runtime_test_world();
    {
        let player = world.get_mut::<AnimationPlayerComponent>(entity).unwrap();
        player.playback_speed = f32::NAN;
        player.time_seconds = f32::NAN;
        player.weight = f32::NAN;
    }
    {
        let player = world
            .get_mut::<AnimationSequencePlayerComponent>(entity)
            .unwrap();
        player.playback_speed = f32::NAN;
        player.time_seconds = f32::NAN;
    }
    for (path, value) in compiled_animation_runtime_test_cases() {
        if !matches!(value, ScenePropertyValue::Scalar(_)) {
            continue;
        }
        let writer = world
            .compile_scene_property_writer(
                &EntityPath::parse("Target").unwrap(),
                &ComponentPropertyPath::parse(path).unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(world.read_compiled_scene_property(&writer),
                Some(ScenePropertyValue::Scalar(current)) if current.is_nan()),
            "{path}"
        );
        let generation = world.world_generation();
        assert!(
            world
                .write_compiled_scene_property(&writer, value.clone())
                .unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(value.clone()),
            "{path}"
        );
        assert!(
            !world.write_compiled_scene_property(&writer, value).unwrap(),
            "{path}"
        );
        assert_eq!(world.world_generation(), generation + 1, "{path}");
    }
}

#[cfg(feature = "animation")]
#[test]
fn compiled_sequence_repeated_animation_runtime_sample_preserves_world_generation() {
    use crate::animation::{apply_compiled_sequence_to_world, compile_sequence_for_world};
    use crate::core::framework::animation::compiler::sequence::compile_animation_sequence;
    use crate::core::framework::animation::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
        AnimationSequenceTrackAsset,
    };
    use crate::core::framework::scene::ScenePropertyValue;

    let (mut world, _) = compiled_animation_runtime_test_world();
    let property_path = ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap();
    let writer = world
        .compile_scene_property_writer(&EntityPath::parse("Target").unwrap(), &property_path)
        .unwrap()
        .unwrap();
    let sequence = AnimationSequenceAsset {
        name: Some("Repeated runtime field".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Target").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path,
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(0.75),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };
    let source = compile_animation_sequence(&sequence)
        .into_artifact()
        .unwrap();
    let compiled = compile_sequence_for_world(&mut world, source).unwrap();
    assert!(compiled.missing_tracks().is_empty());
    let projection_before = world.node_records();
    let generation = world.world_generation();

    let first = apply_compiled_sequence_to_world(&mut world, &compiled, 0.0, false).unwrap();
    assert_eq!(first.applied_tracks, 1);
    assert_eq!(first.missing_tracks, 0);
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(ScenePropertyValue::Scalar(0.75))
    );
    let projection_after = world.node_records();
    assert_ne!(projection_after, projection_before);

    for time in [0.0, 0.5, 0.75] {
        let repeated =
            apply_compiled_sequence_to_world(&mut world, &compiled, time, false).unwrap();
        assert_eq!(repeated.applied_tracks, 1);
        assert_eq!(repeated.missing_tracks, 0);
        assert_eq!(world.world_generation(), generation + 1);
        assert_eq!(world.node_records(), projection_after);
        assert!(compiled.is_current_for(&world));
    }

    let entity = writer.entity();
    world.rename_node(entity, "Renamed target").unwrap();
    assert!(!compiled.is_current_for(&world));
    let stale_generation = world.world_generation();
    let stale_projection = world.node_records();
    let stale = apply_compiled_sequence_to_world(&mut world, &compiled, 0.0, false).unwrap();
    assert_eq!(stale.applied_tracks, 0);
    assert_eq!(stale.missing_tracks, 1);
    assert_eq!(world.world_generation(), stale_generation);
    assert_eq!(world.node_records(), stale_projection);
}
const COMPILED_VM_DYNAMIC_TYPE_PATH: &str = "weather.Component.CompiledVm";

fn compiled_vm_dynamic_test_world() -> (World, crate::scene::EntityId) {
    use crate::scene::reflect::VmTypeBacking;
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectFieldInfo, ReflectSerializationStrategy, ReflectTypeInfo,
        ReflectTypePath, ReflectTypeRegistration,
    };

    let registration = ReflectTypeRegistration::new(
        ReflectTypePath::new(COMPILED_VM_DYNAMIC_TYPE_PATH, "CompiledVm").unwrap(),
        "Compiled VM",
        ReflectTypeInfo::json_with_fields(vec![
            ReflectFieldInfo::from_stable_keys(
                COMPILED_VM_DYNAMIC_TYPE_PATH,
                "coverage",
                "coverage",
                "Scalar",
                ReflectEditorHint::Scalar,
            ),
            ReflectFieldInfo::from_stable_keys(
                COMPILED_VM_DYNAMIC_TYPE_PATH,
                "label",
                "label",
                "String",
                ReflectEditorHint::String,
            )
            .with_editable(false),
        ]),
        ReflectSerializationStrategy::Json,
    )
    .as_component()
    .with_plugin_id("weather")
    .unwrap();
    let mut world = World::empty();
    world
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.rename_node(entity, "Target").unwrap();
    world
        .set_dynamic_component(
            entity,
            COMPILED_VM_DYNAMIC_TYPE_PATH,
            json!({ "coverage": 0.25, "label": "x".repeat(256 * 1024) }),
        )
        .unwrap();
    (world, entity)
}

#[test]
fn compiled_vm_dynamic_repeated_field_write_preserves_large_component() {
    use crate::core::framework::scene::ScenePropertyValue;

    let (mut world, entity) = compiled_vm_dynamic_test_world();
    let path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.coverage")).unwrap();
    let writer = world
        .compile_scene_property_writer(&EntityPath::parse("Target").unwrap(), &path)
        .unwrap()
        .unwrap();
    let component = world
        .dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH)
        .unwrap()
        .clone();
    assert_eq!(component["label"].as_str().unwrap().len(), 256 * 1024);
    let generation = world.world_generation();
    let schema_generation = world.component_type_schema_generation(COMPILED_VM_DYNAMIC_TYPE_PATH);
    let projection = world.node_records();

    for _ in 0..4 {
        assert!(!world
            .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.25))
            .unwrap());
        assert_eq!(world.world_generation(), generation);
        assert_eq!(
            world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
            Some(&component)
        );
        assert_eq!(
            world.read_compiled_scene_property(&writer),
            Some(ScenePropertyValue::Scalar(0.25))
        );
        assert_eq!(
            world.component_type_schema_generation(COMPILED_VM_DYNAMIC_TYPE_PATH),
            schema_generation
        );
        assert!(writer.is_current_for(&world));
        assert_eq!(world.node_records(), projection);
    }
}

#[test]
fn compiled_vm_dynamic_changed_field_write_remains_transactional() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;
    use zircon_runtime_interface::reflect::ReflectError;

    let (mut world, entity) = compiled_vm_dynamic_test_world();
    let path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.coverage")).unwrap();
    let writer = world
        .compile_scene_property_writer(&EntityPath::parse("Target").unwrap(), &path)
        .unwrap()
        .unwrap();
    let mut expected = world
        .dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH)
        .unwrap()
        .clone();
    let generation = world.world_generation();
    let schema_generation = world.component_type_schema_generation(COMPILED_VM_DYNAMIC_TYPE_PATH);

    assert!(world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    expected["coverage"] = json!(0.75);
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&expected)
    );
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(ScenePropertyValue::Scalar(0.75))
    );
    let projection = world.node_records();

    for (value, actual_kind) in [
        (
            ScenePropertyValue::String("invalid".to_string()),
            "JsonString",
        ),
        (ScenePropertyValue::Bool(true), "JsonBool"),
    ] {
        let error = world
            .write_compiled_scene_property(&writer, value)
            .unwrap_err();
        assert!(matches!(
            error,
            SceneError::Reflect(ReflectError::TypeMismatch {
                type_path,
                field_name,
                expected: expected_kind,
                actual,
            }) if type_path == COMPILED_VM_DYNAMIC_TYPE_PATH
                && field_name == "coverage"
                && expected_kind == "Scalar"
                && actual == actual_kind
        ));
        assert_eq!(world.world_generation(), generation + 1);
        assert_eq!(
            world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
            Some(&expected)
        );
        assert_eq!(world.node_records(), projection);
    }

    let mut invalid_component = expected.clone();
    invalid_component["label"] = json!(false);
    let error = world
        .set_dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH, invalid_component)
        .unwrap_err();
    assert!(matches!(
        error,
        SceneError::Reflect(ReflectError::TypeMismatch {
            type_path,
            field_name,
            expected: expected_kind,
            actual,
        }) if type_path == COMPILED_VM_DYNAMIC_TYPE_PATH
            && field_name == "label"
            && expected_kind == "String"
            && actual == "JsonBool"
    ));
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&expected)
    );
    assert!(!world
        .write_compiled_scene_property(&writer, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert_eq!(world.world_generation(), generation + 1);
    assert_eq!(
        world.component_type_schema_generation(COMPILED_VM_DYNAMIC_TYPE_PATH),
        schema_generation
    );
    assert!(writer.is_current_for(&world));
    assert_eq!(world.node_records(), projection);
}

#[test]
fn compiled_vm_dynamic_writer_preserves_error_priority_and_stale_admission() {
    use crate::core::framework::scene::ScenePropertyValue;
    use crate::scene::SceneError;
    use zircon_runtime_interface::reflect::ReflectError;

    let (mut world, entity) = compiled_vm_dynamic_test_world();
    let missing = world.spawn_node(NodeKind::Empty).unwrap();
    world.rename_node(missing, "Missing").unwrap();
    let coverage_path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.coverage")).unwrap();
    let label_path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.label")).unwrap();
    let unknown_path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.unknown")).unwrap();
    let target = EntityPath::parse("Target").unwrap();
    let missing_path = EntityPath::parse("Missing").unwrap();
    let coverage_writer = world
        .compile_scene_property_writer(&target, &coverage_path)
        .unwrap()
        .unwrap();
    let label_writer = world
        .compile_scene_property_writer(&target, &label_path)
        .unwrap()
        .unwrap();
    let unknown_writer = world
        .compile_scene_property_writer(&target, &unknown_path)
        .unwrap()
        .unwrap();
    let missing_writer = world
        .compile_scene_property_writer(&missing_path, &coverage_path)
        .unwrap()
        .unwrap();
    let missing_label_writer = world
        .compile_scene_property_writer(&missing_path, &label_path)
        .unwrap()
        .unwrap();
    let missing_unknown_writer = world
        .compile_scene_property_writer(&missing_path, &unknown_path)
        .unwrap()
        .unwrap();
    let component = world
        .dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH)
        .unwrap()
        .clone();
    let label = component["label"].as_str().unwrap();
    let generation = world.world_generation();
    let projection = world.node_records();

    for writer in [&label_writer, &missing_label_writer] {
        for value in [
            ScenePropertyValue::String(label.to_string()),
            ScenePropertyValue::Enum(label.to_string()),
        ] {
            let error = world
                .write_compiled_scene_property(writer, value)
                .unwrap_err();
            assert!(matches!(
                error,
                SceneError::NonEditableDynamicComponentProperty { component_id, property }
                    if component_id == COMPILED_VM_DYNAMIC_TYPE_PATH && property == "label"
            ));
        }
    }
    for writer in [&unknown_writer, &missing_unknown_writer] {
        let error = world
            .write_compiled_scene_property(writer, ScenePropertyValue::Scalar(0.25))
            .unwrap_err();
        assert!(matches!(
            error,
            SceneError::UndeclaredDynamicComponentProperty { component_id, property }
                if component_id == COMPILED_VM_DYNAMIC_TYPE_PATH && property == "unknown"
        ));
    }
    for value in [
        ScenePropertyValue::Scalar(0.25),
        ScenePropertyValue::Enum("not a scalar".to_string()),
    ] {
        let error = world
            .write_compiled_scene_property(&missing_writer, value)
            .unwrap_err();
        assert!(matches!(
            error,
            SceneError::Reflect(ReflectError::MissingComponent { entity: actual, type_path })
                if actual == missing && type_path == COMPILED_VM_DYNAMIC_TYPE_PATH
        ));
    }
    let error = world
        .write_compiled_scene_property(
            &coverage_writer,
            ScenePropertyValue::Enum("not a scalar".to_string()),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        SceneError::Reflect(ReflectError::TypeMismatch {
            type_path,
            field_name,
            expected,
            actual,
        }) if type_path == COMPILED_VM_DYNAMIC_TYPE_PATH
            && field_name == "coverage"
            && expected == "Scalar"
            && actual == "JsonString"
    ));
    for (writer, path) in [
        (&coverage_writer, &coverage_path),
        (&label_writer, &label_path),
        (&unknown_writer, &unknown_path),
        (&missing_writer, &coverage_path),
        (&missing_label_writer, &label_path),
        (&missing_unknown_writer, &unknown_path),
    ] {
        for value in [
            ScenePropertyValue::Scalar(f32::NAN),
            ScenePropertyValue::Scalar(f32::INFINITY),
            ScenePropertyValue::Vec3([0.0, f32::NEG_INFINITY, 0.0]),
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ] {
            let error = world
                .write_compiled_scene_property(writer, value)
                .unwrap_err();
            assert!(matches!(
                error,
                SceneError::DynamicComponentPropertyUnsupportedValue { property_path }
                    if property_path == path.to_string()
            ));
        }
    }
    assert_eq!(world.world_generation(), generation);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&component)
    );
    assert_eq!(
        world.dynamic_component(missing, COMPILED_VM_DYNAMIC_TYPE_PATH),
        None
    );
    assert_eq!(world.node_records(), projection);

    world.rename_node(entity, "Renamed target").unwrap();
    let stale_generation = world.world_generation();
    let stale_projection = world.node_records();
    for (writer, path) in [
        (&coverage_writer, &coverage_path),
        (&label_writer, &label_path),
        (&unknown_writer, &unknown_path),
    ] {
        assert!(!writer.is_current_for(&world));
        for value in [
            ScenePropertyValue::Scalar(0.25),
            ScenePropertyValue::Scalar(f32::NAN),
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ] {
            let error = world
                .write_compiled_scene_property(writer, value)
                .unwrap_err();
            assert!(matches!(
                error,
                SceneError::PropertyUnavailable { entity: actual, property_path }
                    if actual == entity && property_path == path.to_string()
            ));
        }
    }
    assert_eq!(world.world_generation(), stale_generation);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&component)
    );
    assert_eq!(world.node_records(), stale_projection);
}

#[cfg(feature = "animation")]
#[test]
fn compiled_sequence_repeated_vm_dynamic_sample_preserves_world_generation() {
    use crate::animation::{apply_compiled_sequence_to_world, compile_sequence_for_world};
    use crate::core::framework::animation::compiler::sequence::compile_animation_sequence;
    use crate::core::framework::animation::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
        AnimationSequenceTrackAsset,
    };
    use crate::core::framework::scene::ScenePropertyValue;

    let (mut world, entity) = compiled_vm_dynamic_test_world();
    let property_path =
        ComponentPropertyPath::parse(&format!("{COMPILED_VM_DYNAMIC_TYPE_PATH}.coverage")).unwrap();
    let writer = world
        .compile_scene_property_writer(&EntityPath::parse("Target").unwrap(), &property_path)
        .unwrap()
        .unwrap();
    let mut expected = world
        .dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH)
        .unwrap()
        .clone();
    let sequence = AnimationSequenceAsset {
        name: Some("Repeated VM dynamic field".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Target").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path,
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![AnimationChannelKeyAsset {
                        time_seconds: 0.0,
                        value: AnimationChannelValueAsset::Scalar(0.75),
                        in_tangent: None,
                        out_tangent: None,
                    }],
                },
            }],
        }],
    };
    let source = compile_animation_sequence(&sequence)
        .into_artifact()
        .unwrap();
    let compiled = compile_sequence_for_world(&mut world, source).unwrap();
    assert!(compiled.missing_tracks().is_empty());
    let generation = world.world_generation();

    let first = apply_compiled_sequence_to_world(&mut world, &compiled, 0.0, false).unwrap();
    assert_eq!(first.applied_tracks, 1);
    assert_eq!(first.missing_tracks, 0);
    assert_eq!(world.world_generation(), generation + 1);
    expected["coverage"] = json!(0.75);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&expected)
    );
    assert_eq!(
        world.read_compiled_scene_property(&writer),
        Some(ScenePropertyValue::Scalar(0.75))
    );
    let projection = world.node_records();

    for time in [0.0, 0.5, 0.75] {
        let repeated =
            apply_compiled_sequence_to_world(&mut world, &compiled, time, false).unwrap();
        assert_eq!(repeated.applied_tracks, 1);
        assert_eq!(repeated.missing_tracks, 0);
        assert_eq!(world.world_generation(), generation + 1);
        assert_eq!(
            world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
            Some(&expected)
        );
        assert_eq!(world.node_records(), projection);
        assert!(compiled.is_current_for(&world));
    }

    world.rename_node(entity, "Renamed target").unwrap();
    assert!(!compiled.is_current_for(&world));
    let stale_generation = world.world_generation();
    let stale_projection = world.node_records();
    let stale = apply_compiled_sequence_to_world(&mut world, &compiled, 0.0, false).unwrap();
    assert_eq!(stale.applied_tracks, 0);
    assert_eq!(stale.missing_tracks, 1);
    assert_eq!(world.world_generation(), stale_generation);
    assert_eq!(
        world.dynamic_component(entity, COMPILED_VM_DYNAMIC_TYPE_PATH),
        Some(&expected)
    );
    assert_eq!(world.node_records(), stale_projection);
}
