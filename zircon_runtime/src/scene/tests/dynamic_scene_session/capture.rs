use std::fs;

use crate::scene::{RuntimeSessionArchive, RuntimeSessionMetadata, World};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_capture_world_slot_to_missing_path_creates_archive_atomically() {
    let mut source = World::empty();
    source.spawn_node(crate::scene::NodeKind::Mesh);
    let root = unique_temp_root("runtime_session_capture_world_missing_path");
    let path = root.join("sessions").join("archive.zrsession.json");

    let manifest = RuntimeSessionArchive::capture_world_slot_to_path_atomically(
        &path,
        " autosave ",
        &source,
        RuntimeSessionMetadata::default()
            .with_display_name("Autosave")
            .with_tag(" autosave ")
            .with_updated_at_unix_millis(100),
    )
    .expect("missing session archive path should be created atomically");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["autosave"]);
    assert_eq!(
        manifest
            .latest_updated_slot_with_tag("autosave")
            .map(|slot| slot.slot_id.as_str()),
        Some("autosave")
    );

    let loaded =
        RuntimeSessionArchive::load_from_path(&path).expect("created archive should reload");
    assert_eq!(
        loaded
            .manifest()
            .expect("loaded archive should project manifest")
            .slot("autosave")
            .expect("autosave summary should exist")
            .entity_count,
        1
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_capture_level_slot_to_existing_path_upserts_and_preserves_other_slots() {
    let source = World::empty();
    let existing = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "autosave", "autosave", 20),
        tagged_slot(&source, "manual", "manual", 10),
    ])
    .expect("existing archive should validate");
    let root = unique_temp_root("runtime_session_capture_level_existing_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    existing
        .save_to_path_atomically(&path)
        .expect("existing archive should save before level capture");

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_default_level();
    level.with_world_mut(|world| {
        world.spawn_node(crate::scene::NodeKind::Camera);
    });
    level.set_metadata(crate::scene::LevelMetadata {
        project_root: Some("project".to_string()),
        asset_uri: Some("res://levels/main.zscene".to_string()),
        display_name: Some("Captured Level".to_string()),
    });

    let manifest =
        RuntimeSessionArchive::capture_level_slot_to_path_atomically(&path, " manual ", &level)
            .expect("existing archive slot should upsert from level");

    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    let summary = manifest.slot("manual").expect("manual slot should remain");
    assert_eq!(summary.entity_count, 1);
    assert_eq!(
        summary.metadata.asset_uri.as_deref(),
        Some("res://levels/main.zscene")
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("upserted archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_capture_to_path_rejects_invalid_existing_archive_without_overwrite() {
    let source = World::empty();
    let root = unique_temp_root("runtime_session_capture_invalid_existing_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    fs::create_dir_all(path.parent().expect("session path should have parent"))
        .expect("session directory should be created");
    fs::write(&path, "{ invalid-json").expect("invalid archive fixture should write");

    let result = RuntimeSessionArchive::capture_world_slot_to_path_atomically(
        &path,
        "autosave",
        &source,
        RuntimeSessionMetadata::default().with_tag("autosave"),
    );

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(&path).expect("invalid archive should remain readable"),
        "{ invalid-json"
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
