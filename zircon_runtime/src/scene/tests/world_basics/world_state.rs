use super::*;

#[test]
fn project_loader_deserializes_the_world_payload_once() {
    let source = include_str!("../../world/project_io/document.rs");
    let load_start = source
        .find("pub fn load_project_from_path")
        .expect("project load entry point");
    let load_end = source[load_start..]
        .find("pub(super) fn normalize_scene_asset_after_load")
        .map(|offset| load_start + offset)
        .expect("project load boundary");
    let load = &source[load_start..load_end];

    assert_eq!(
        load.matches("serde_json::from_str(document.world.get())")
            .count(),
        1
    );
    assert!(load.contains("World::from_persistent_state(persisted_state)"));
    assert!(!load.contains("let mut world: World = serde_json::from_str"));
}

#[test]
fn world_bootstraps_with_renderable_defaults() {
    let world = World::new();
    let snapshot = world.to_render_snapshot();

    assert!(!snapshot.scene.meshes.is_empty());
    assert!(snapshot.overlays.grid.is_none());
    assert!(snapshot.overlays.highlights.is_none());
    assert!(snapshot.overlays.selection_anchors.is_empty());
    assert!(snapshot.overlays.handles.is_empty());
    assert!(snapshot.overlays.scene_gizmos.is_empty());
    assert_eq!(
        world.schedule().stages(),
        vec![
            SystemStage::First,
            SystemStage::PreUpdate,
            SystemStage::FixedFirst,
            SystemStage::FixedUpdate,
            SystemStage::FixedPostUpdate,
            SystemStage::Update,
            SystemStage::PostUpdate,
            SystemStage::Last,
            SystemStage::RenderExtract,
        ]
    );
}

#[test]
fn spawned_entities_have_unique_ids() {
    let mut world = World::new();
    let first = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    assert_ne!(first, second);
}

#[test]
fn spawn_node_assigns_one_based_kind_ordinals() {
    let mut world = World::empty();
    let first_mesh = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let second_mesh = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let first_cube = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");

    assert_eq!(world.get::<Name>(first_mesh).unwrap().0, "Mesh 1");
    assert_eq!(world.get::<Name>(second_mesh).unwrap().0, "Mesh 2");
    assert_eq!(world.get::<Name>(first_cube).unwrap().0, "Cube 1");
}

#[test]
fn spawn_node_kind_ordinals_survive_removal_and_world_deserialization() {
    let mut world = World::empty();
    let first = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let removed = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.remove_entity(removed).unwrap();
    let replacement = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    assert_eq!(world.get::<Name>(replacement).unwrap().0, "Mesh 2");

    let encoded = serde_json::to_string(&world).unwrap();
    let mut restored: World = serde_json::from_str(&encoded).unwrap();
    let after_restore = restored
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");

    assert_eq!(world.get::<Name>(first).unwrap().0, "Mesh 1");
    assert_eq!(restored.get::<Name>(after_restore).unwrap().0, "Mesh 3");
}

#[test]
fn hierarchy_updates_world_transform() {
    let mut world = World::new();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn local_transform_reads_one_component_without_projecting_a_scene_node() {
    let mut world = World::new();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let parent_transform = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
    let child_transform = Transform::from_translation(Vec3::new(2.0, 0.0, 0.0));
    world.update_transform(parent, parent_transform).unwrap();
    world.update_transform(child, child_transform).unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    assert_eq!(world.local_transform(child), Some(child_transform));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn update_transform_rejects_values_that_cannot_be_persisted() {
    let mut world = World::new();
    let entity = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let original = world.local_transform(entity).unwrap();

    let mut zero_scale = original;
    zero_scale.scale.x = 0.0;
    assert!(matches!(
        world.update_transform(entity, zero_scale),
        Err(crate::scene::SceneError::ZeroScaleTransform { entity: error_entity, axis: "x" })
            if error_entity == entity
    ));
    assert_eq!(world.local_transform(entity), Some(original));

    let mut zero_rotation = original;
    zero_rotation.rotation = crate::core::math::Quat::from_array([0.0; 4]);
    assert!(matches!(
        world.update_transform(entity, zero_rotation),
        Err(crate::scene::SceneError::ZeroLengthQuaternion { .. })
    ));
    assert_eq!(world.local_transform(entity), Some(original));
}

#[test]
fn project_load_rejects_invalid_orphan_local_transform() {
    let world = World::new();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_invalid_scene_{unique}.json"));
    world.save_project_to_path(&path).unwrap();
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let transforms = document["world"]["local_transforms"]
        .as_object_mut()
        .expect("serialized project world must contain local transforms");
    let mut orphan = transforms
        .values()
        .next()
        .expect("bootstrap world must contain a local transform")
        .clone();
    orphan["transform"]["scale"] = serde_json::json!([0.0, 1.0, 1.0]);
    transforms.insert("999999".to_string(), orphan);
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    assert!(matches!(
        World::load_project_from_path(&path),
        Err(crate::scene::world::SceneProjectError::Scene(
            crate::scene::SceneError::ZeroScaleTransform {
                entity: 999_999,
                axis: "x"
            }
        ))
    ));

    fs::remove_file(path).unwrap();
}

#[test]
fn project_load_rejects_valid_orphan_component_without_panicking() {
    let world = World::new();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_orphan_scene_{unique}.json"));
    world.save_project_to_path(&path).unwrap();

    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let transforms = document["world"]["local_transforms"]
        .as_object_mut()
        .expect("serialized project world must contain local transforms");
    let orphan = transforms
        .values()
        .next()
        .expect("bootstrap world must contain a local transform")
        .clone();
    transforms.insert("999999".to_string(), orphan);
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    assert!(matches!(
        World::load_project_from_path(&path),
        Err(crate::scene::world::SceneProjectError::Scene(
            crate::scene::SceneError::MissingEntity {
                operation: "load persisted component",
                entity: 999_999
            }
        ))
    ));

    fs::remove_file(path).unwrap();
}

#[test]
fn project_load_rejects_exhausted_entity_ids_without_panicking() {
    let world = World::empty();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_exhausted_scene_{unique}.json"));
    world.save_project_to_path(&path).unwrap();

    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["world"]["entities"] = serde_json::json!([u64::MAX]);
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    assert!(matches!(
        World::load_project_from_path(&path),
        Err(crate::scene::world::SceneProjectError::ProjectNormalization {
            path: error_path,
            source: crate::scene::SceneError::EntityIdExhausted { entity: u64::MAX },
        }) if error_path == path
    ));

    fs::remove_file(path).unwrap();
}

#[test]
fn project_load_rejects_default_node_allocation_exhaustion_without_panicking() {
    let world = World::empty();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    for max_entity in [u64::MAX - 3, u64::MAX - 2] {
        let path = std::env::temp_dir().join(format!(
            "zircon_default_node_allocation_exhaustion_{unique}_{max_entity}.json"
        ));
        world.save_project_to_path(&path).unwrap();

        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["world"]["entities"] = serde_json::json!([max_entity]);
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

        assert!(matches!(
            World::load_project_from_path(&path),
            Err(crate::scene::world::SceneProjectError::ProjectNormalization {
                path: error_path,
                source: crate::scene::SceneError::EntityIdExhausted { entity: u64::MAX },
            }) if error_path == path
        ));

        fs::remove_file(path).unwrap();
    }
}

#[test]
fn project_load_rejects_an_unsupported_project_format_version() {
    let world = World::new();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_future_scene_{unique}.json"));
    world.save_project_to_path(&path).unwrap();

    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["format_version"] = serde_json::json!(3);
    document["world"] = serde_json::json!({
        "future_world_shape": {
            "entity_storage": "incompatible-with-v2"
        }
    });
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    assert!(matches!(
        World::load_project_from_path(&path),
        Err(
            crate::scene::world::SceneProjectError::UnsupportedProjectFormatVersion {
                expected: 2,
                actual: 3,
            }
        )
    ));

    fs::remove_file(path).unwrap();
}

#[test]
fn project_roundtrip_preserves_imported_meshes() {
    let mut world = World::new();
    let imported = world
        .spawn_mesh_node(
            model_handle("res://models/robot.obj"),
            material_handle("res://materials/robot.zmaterial"),
        )
        .expect("test mesh spawn should succeed");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_scene_roundtrip_{unique}.json"));
    world.save_project_to_path(&path).unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_text_excludes_authoring_tokens(
        "world project serialization",
        &saved,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let imported_node = loaded.find_node(imported).unwrap();
    assert!(matches!(imported_node.kind, NodeKind::Mesh));
    assert_eq!(
        imported_node.mesh.as_ref().unwrap().model,
        model_handle("res://models/robot.obj")
    );
}

#[test]
fn node_record_roundtrip_restores_same_entity() {
    let mut world = World::new();
    let cube = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let record = world.node_record(cube).unwrap();

    world.remove_entity(cube).unwrap();
    assert!(!world.contains_entity(cube));

    world.insert_node_record(record.clone()).unwrap();

    let restored = world.node_record(cube).unwrap();
    assert_eq!(restored, record);
}

#[test]
fn recursive_remove_returns_parent_and_children_records() {
    let mut world = World::new();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(child, Some(parent)).unwrap();

    let removed = world.remove_entity_recursive(parent).unwrap();
    assert_eq!(removed.len(), 2);
    assert!(!world.contains_entity(parent));
    assert!(!world.contains_entity(child));
}

#[test]
fn set_parent_checked_rejects_hierarchy_cycles() {
    let mut world = World::new();
    let parent = world
        .spawn_node(NodeKind::Cube)
        .expect("test scene spawn should succeed");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    world.set_parent_checked(child, Some(parent)).unwrap();

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.to_string().contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
}
