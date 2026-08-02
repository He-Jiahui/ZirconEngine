use std::fs;

use crate::scene::{
    LevelMetadata, RuntimeSessionArchive, RuntimeSessionMetadata, RuntimeSessionSlot, World,
};

use super::{tagged_slot, unique_temp_root};

#[test]
fn runtime_session_archive_restores_slot_from_path_to_empty_world() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(crate::scene::NodeKind::Mesh);
    source
        .rename_node(saved_entity, "Path Restored Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual",
            &source,
            RuntimeSessionMetadata::default().with_display_name("Manual"),
        )
        .expect("manual slot should capture"),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_restore_path_to_world");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path restore");

    let restored = RuntimeSessionArchive::restore_slot_from_path_to_empty_world(&path, "manual")
        .expect("slot should restore directly from archive path");

    assert_eq!(
        restored
            .find_node(saved_entity)
            .expect("restored entity should keep source id")
            .name,
        "Path Restored Mesh"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_restores_slot_from_path_into_level_and_applies_metadata() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(crate::scene::NodeKind::Camera);
    source
        .rename_node(saved_entity, "Path Restored Camera")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "level",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Loaded From Path")
                .with_tag("manual"),
        )
        .expect("level slot should capture"),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_restore_path_to_level");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before level restore");

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_default_level();
    let stale_entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(crate::scene::NodeKind::Mesh);
        world
            .rename_node(entity, "Stale Mesh")
            .expect("stale entity should be named");
        entity
    });

    let report = RuntimeSessionArchive::restore_slot_from_path_into_level(&path, "level", &level)
        .expect("slot should restore from path into level");

    assert_eq!(report.slot_id, "level");
    assert_eq!(report.entity_count, 1);
    assert_eq!(
        level.metadata().display_name.as_deref(),
        Some("Loaded From Path")
    );
    level.with_world(|world| {
        assert!(world.find_node(stale_entity).is_none());
        assert_eq!(
            world
                .find_node(saved_entity)
                .expect("restored entity should exist")
                .name,
            "Path Restored Camera"
        );
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_applies_slot_from_path_to_live_world_and_level() {
    let mut source = World::empty();
    let source_entity = source.spawn_node(crate::scene::NodeKind::Mesh);
    source
        .rename_node(source_entity, "Path Instanced Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world("prefab", &source).expect("prefab slot should capture"),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_apply_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before apply");

    let mut world = World::empty();
    let existing_world_entity = world.spawn_node(crate::scene::NodeKind::Camera);
    assert_eq!(existing_world_entity, source_entity);
    let world_remap =
        RuntimeSessionArchive::apply_slot_from_path_to_world(&path, "prefab", &mut world)
            .expect("slot should apply from path into live world");
    let mapped_world_entity = world_remap
        .get(source_entity)
        .expect("source entity should be remapped");
    assert_ne!(mapped_world_entity, source_entity);
    assert_eq!(
        world
            .find_node(mapped_world_entity)
            .expect("mapped entity should exist in live world")
            .name,
        "Path Instanced Mesh"
    );

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    let existing_level_entity =
        level.with_world_mut(|world| world.spawn_node(crate::scene::NodeKind::Camera));
    assert_eq!(existing_level_entity, source_entity);
    let level_remap = RuntimeSessionArchive::apply_slot_from_path_to_level(&path, "prefab", &level)
        .expect("slot should apply from path into live level");
    let mapped_level_entity = level_remap
        .get(source_entity)
        .expect("source entity should be remapped for level");
    level.with_world(|world| {
        assert!(world.find_node(existing_level_entity).is_some());
        assert_eq!(
            world
                .find_node(mapped_level_entity)
                .expect("mapped level entity should exist")
                .name,
            "Path Instanced Mesh"
        );
    });

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_path_load_helpers_report_missing_slot() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_missing_slot");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before missing-slot load");

    let result = RuntimeSessionArchive::restore_slot_from_path_to_empty_world(&path, "missing");

    assert!(matches!(
        result,
        Err(crate::scene::RuntimeSessionArchiveError::MissingSlot { .. })
    ));

    let _ = fs::remove_dir_all(root);
}
