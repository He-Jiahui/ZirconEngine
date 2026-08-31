use super::*;

#[test]
fn runtime_session_archive_restores_slot_into_level_and_resets_runtime_state() {
    let mut source = World::empty();
    let saved_entity = source
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    source
        .rename_node(saved_entity, "Restored Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_world_with_metadata(
        "level-slot",
        &source,
        RuntimeSessionMetadata::default()
            .with_display_name("Restored Level")
            .with_tag("level"),
    )
    .expect("archive should capture level slot");

    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    let stale_entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Camera)
            .expect("test scene spawn should succeed");
        world
            .rename_node(entity, "Stale Camera")
            .expect("stale entity should be named");
        entity
    });
    assert!(level.record_physics_step_if_replacement_epoch(
        level.capture_world_replacement_epoch(),
        PhysicsWorldStepPlan {
            steps: 1,
            step_seconds: 0.016,
            remaining_seconds: 0.0,
            interpolation_alpha: 0.0,
        },
        Vec::new(),
        Vec::new(),
    ));

    let report = archive
        .restore_slot_into_level("level-slot", &level)
        .expect("slot should restore directly into the level");

    assert_eq!(report.slot_id, "level-slot");
    assert_eq!(report.entity_count, 1);
    assert_eq!(
        level.metadata().display_name.as_deref(),
        Some("Restored Level")
    );
    assert!(level.last_physics_step_plan().is_none());
    level.with_world(|world| {
        assert!(world.find_node(stale_entity).is_none());
        assert_eq!(
            world
                .find_node(saved_entity)
                .expect("restored entity should exist")
                .name,
            "Restored Mesh"
        );
    });
}

#[test]
fn runtime_session_archive_applies_slot_to_live_level_with_entity_remap() {
    let mut source = World::empty();
    let saved_entity = source
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    source
        .rename_node(saved_entity, "Instanced Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_world("prefab-slot", &source)
        .expect("archive should capture prefab slot");

    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), Default::default());
    let existing_entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Camera)
            .expect("test scene spawn should succeed");
        world
            .rename_node(entity, "Live Camera")
            .expect("existing entity should be named");
        entity
    });
    assert_eq!(existing_entity, saved_entity);

    let remap = archive
        .apply_slot_to_level("prefab-slot", &level)
        .expect("slot should apply into the live level");
    let mapped_entity = remap
        .get(saved_entity)
        .expect("source entity should be remapped after id collision");

    assert_ne!(mapped_entity, existing_entity);
    level.with_world(|world| {
        assert_eq!(
            world
                .find_node(existing_entity)
                .expect("existing entity should remain")
                .name,
            "Live Camera"
        );
        assert_eq!(
            world
                .find_node(mapped_entity)
                .expect("mapped entity should be spawned")
                .name,
            "Instanced Mesh"
        );
    });
}
