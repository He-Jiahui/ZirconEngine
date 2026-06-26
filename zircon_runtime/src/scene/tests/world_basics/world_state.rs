use super::*;

#[test]
fn world_bootstraps_with_renderable_defaults() {
    let world = World::new();
    let snapshot = world.to_render_snapshot();

    assert!(!snapshot.scene.meshes.is_empty());
    assert!(snapshot.overlays.grid.is_none());
    assert!(snapshot.overlays.selection.is_empty());
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
    let first = world.spawn_node(NodeKind::Cube);
    let second = world.spawn_node(NodeKind::Cube);
    assert_ne!(first, second);
}

#[test]
fn spawn_node_assigns_one_based_kind_ordinals() {
    let mut world = World::empty();
    let first_mesh = world.spawn_node(NodeKind::Mesh);
    let second_mesh = world.spawn_node(NodeKind::Mesh);
    let first_cube = world.spawn_node(NodeKind::Cube);

    assert_eq!(world.get::<Name>(first_mesh).unwrap().0, "Mesh 1");
    assert_eq!(world.get::<Name>(second_mesh).unwrap().0, "Mesh 2");
    assert_eq!(world.get::<Name>(first_cube).unwrap().0, "Cube 1");
}

#[test]
fn hierarchy_updates_world_transform() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
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
fn project_roundtrip_preserves_imported_meshes() {
    let mut world = World::new();
    let imported = world.spawn_mesh_node(
        model_handle("res://models/robot.obj"),
        material_handle("res://materials/robot.zmaterial"),
    );

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
    let cube = world.spawn_node(NodeKind::Cube);
    let record = world.node_record(cube).unwrap();

    assert!(world.remove_entity(cube));
    assert!(!world.contains_entity(cube));

    world.insert_node_record(record.clone()).unwrap();

    let restored = world.node_record(cube).unwrap();
    assert_eq!(restored, record);
}

#[test]
fn recursive_remove_returns_parent_and_children_records() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let removed = world.remove_entity_recursive(parent);
    assert_eq!(removed.len(), 2);
    assert!(!world.contains_entity(parent));
    assert!(!world.contains_entity(child));
}

#[test]
fn set_parent_checked_rejects_hierarchy_cycles() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.to_string().contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
}
