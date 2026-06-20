use std::fs;

use crate::scene::{
    LevelMetadata, NodeKind, RuntimeSessionArchive, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionMetadata, RuntimeSessionSlot, RuntimeSessionSlotSelector, World,
};

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
fn runtime_session_archive_previews_world_capture_without_mutating_archive() {
    let empty = World::empty();
    let mut source = World::empty();
    source.spawn_node(crate::scene::NodeKind::Mesh);
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&empty, "autosave", "autosave", 20)])
            .expect("existing archive should validate");

    let report = archive
        .preview_capture_world_slot(
            " autosave ",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Autosave Preview")
                .with_tag(" autosave ")
                .with_tag("autosave")
                .with_updated_at_unix_millis(100),
        )
        .expect("world capture preview should validate");

    assert_eq!(report.slot_id, "autosave");
    assert!(report.will_replace_existing);
    assert_eq!(
        report.metadata.display_name.as_deref(),
        Some("Autosave Preview")
    );
    assert_eq!(report.metadata.tags, vec!["autosave"]);
    assert_eq!(report.metadata.updated_at_unix_millis, Some(100));
    assert_eq!(report.entity_count, 1);
    assert_eq!(report.resource_count, 0);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["autosave"]);
    assert_eq!(
        archive
            .slot("autosave")
            .expect("autosave slot should remain")
            .metadata
            .updated_at_unix_millis,
        Some(20)
    );

    let new_slot = archive
        .preview_capture_world_slot(" manual ", &source, RuntimeSessionMetadata::default())
        .expect("new slot capture preview should validate");
    assert_eq!(new_slot.slot_id, "manual");
    assert!(!new_slot.will_replace_existing);
}

#[test]
fn runtime_session_archive_world_capture_commit_matches_preview_generated_slot() {
    let empty = World::empty();
    let mut source = World::empty();
    source.spawn_node(NodeKind::Mesh);
    source.spawn_node(NodeKind::Camera);
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&empty, "manual", "manual", 20)])
            .expect("existing archive should validate");
    let metadata = RuntimeSessionMetadata::default()
        .with_display_name("Preview Commit Parity")
        .with_tag(" manual ")
        .with_updated_at_unix_millis(120);

    let preview = archive
        .preview_capture_world_slot(" manual ", &source, metadata.clone())
        .expect("world capture preview should produce canonical slot plan");

    let mut committed = archive.clone();
    committed
        .capture_world_slot(" manual ", &source, metadata)
        .expect("world capture commit should use preview-generated slot");

    let committed_summary = committed
        .manifest()
        .expect("committed archive should project manifest")
        .slot("manual")
        .expect("manual slot summary should exist after commit")
        .clone();
    assert_eq!(preview.slot_id, "manual");
    assert!(preview.will_replace_existing);
    assert_eq!(committed_summary.slot_id, preview.slot_id);
    assert_eq!(committed_summary.metadata, preview.metadata);
    assert_eq!(committed_summary.entity_count, preview.entity_count);
    assert_eq!(committed_summary.resource_count, preview.resource_count);
    assert_eq!(preview.entity_count, 2);
    assert_eq!(
        archive
            .slot("manual")
            .expect("source archive slot should remain unchanged")
            .metadata
            .updated_at_unix_millis,
        Some(20)
    );
}

#[test]
fn runtime_session_archive_level_capture_preview_preserves_from_level_semantics() {
    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    level.with_world_mut(|world| {
        world.spawn_node(NodeKind::Camera);
        world.spawn_node(NodeKind::Mesh);
    });
    level.set_metadata(LevelMetadata {
        project_root: Some("project".to_string()),
        asset_uri: Some("res://levels/preview.zscene".to_string()),
        display_name: Some("Preview Level".to_string()),
    });
    let archive = RuntimeSessionArchive::from_slots(Vec::<RuntimeSessionSlot>::new())
        .expect("empty archive should validate");
    let expected_slot = RuntimeSessionSlot::from_level(" level-preview ", &level)
        .expect("level slot should capture through canonical constructor");

    let preview = archive
        .preview_capture_level_slot(" level-preview ", &level)
        .expect("level capture preview should use RuntimeSessionSlot::from_level semantics");

    assert_eq!(preview.slot_id, expected_slot.slot_id);
    assert!(!preview.will_replace_existing);
    assert_eq!(preview.metadata, expected_slot.metadata);
    assert_eq!(preview.entity_count, expected_slot.scene.entities.len());
    assert_eq!(preview.resource_count, expected_slot.scene.resources.len());
    assert_eq!(preview.entity_count, 2);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), Vec::<&str>::new());
}

#[test]
fn runtime_session_archive_capture_retention_reuses_shared_preview_report_projection() {
    let empty = World::empty();
    let mut captured_world = World::empty();
    captured_world.spawn_node(NodeKind::Mesh);
    captured_world.spawn_node(NodeKind::PointLight);
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&empty, "manual-new", "manual", 50),
        tagged_slot(&empty, "manual-mid", "manual", 20),
        tagged_slot(&empty, "manual-old", "manual", 10),
        tagged_slot(&empty, "autosave", "autosave", 30),
    ])
    .expect("archive should validate capture-retention fixture slots");
    let metadata = RuntimeSessionMetadata::default()
        .with_display_name("Captured Retention")
        .with_tag(" manual ")
        .with_updated_at_unix_millis(90);

    let capture_preview = archive
        .preview_capture_world_slot(" manual-mid ", &captured_world, metadata.clone())
        .expect("ordinary world capture preview should build shared projection");
    let retention_preview = archive
        .preview_capture_world_slot_with_tag_retention(
            " manual ",
            " manual-mid ",
            &captured_world,
            metadata,
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
        )
        .expect("capture-retention preview should reuse shared capture preview projection");

    assert_eq!(retention_preview.capture, capture_preview);
    assert_eq!(retention_preview.capture.slot_id, "manual-mid");
    assert!(retention_preview.capture.will_replace_existing);
    assert_eq!(retention_preview.capture.entity_count, 2);
    assert_eq!(
        retention_preview.prune.removed_slot_ids,
        vec!["manual-new", "manual-old"]
    );
    assert_eq!(
        retention_preview.manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid", "manual-new", "manual-old"]
    );
    assert_eq!(
        archive
            .slot("manual-mid")
            .expect("source archive slot should remain unchanged")
            .metadata
            .updated_at_unix_millis,
        Some(20)
    );
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
    let level = manager.create_level(World::empty(), LevelMetadata::default());
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
fn runtime_session_archive_previews_capture_to_path_without_writing_archive() {
    let source = World::empty();
    let existing = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "autosave", "autosave", 20),
        tagged_slot(&source, "manual", "manual", 10),
    ])
    .expect("existing archive should validate");
    let root = unique_temp_root("runtime_session_capture_path_preview");
    let path = root.join("sessions").join("archive.zrsession.json");
    existing
        .save_to_path_atomically(&path)
        .expect("existing archive should save before capture preview");
    let payload = fs::read_to_string(&path).expect("existing archive payload should be readable");

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    level.with_world_mut(|world| {
        world.spawn_node(crate::scene::NodeKind::Camera);
    });
    level.set_metadata(crate::scene::LevelMetadata {
        project_root: Some("project".to_string()),
        asset_uri: Some("res://levels/main.zscene".to_string()),
        display_name: Some("Captured Level".to_string()),
    });

    let report =
        RuntimeSessionArchive::preview_capture_level_slot_to_path(&path, " manual ", &level)
            .expect("path level capture preview should validate");

    assert_eq!(report.slot_id, "manual");
    assert!(report.will_replace_existing);
    assert_eq!(report.entity_count, 1);
    assert_eq!(report.resource_count, 0);
    assert_eq!(
        report.metadata.asset_uri.as_deref(),
        Some("res://levels/main.zscene")
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after preview"),
        payload
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should reload after capture preview")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let mut missing_source = World::empty();
    missing_source.spawn_node(crate::scene::NodeKind::PointLight);
    let missing_path = root.join("new").join("archive.zrsession.json");
    let missing_report = RuntimeSessionArchive::preview_capture_world_slot_to_path(
        &missing_path,
        " autosave ",
        &missing_source,
        RuntimeSessionMetadata::default().with_tag("autosave"),
    )
    .expect("missing path capture preview should use an empty archive");
    assert_eq!(missing_report.slot_id, "autosave");
    assert!(!missing_report.will_replace_existing);
    assert_eq!(missing_report.entity_count, 1);
    assert!(!missing_path.exists());

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

#[test]
fn runtime_session_archive_selected_capture_targets_resolved_slot_and_preserves_metadata() {
    let empty = World::empty();
    let mut replacement = World::empty();
    replacement.spawn_node(NodeKind::Mesh);
    let mut preserved_replacement = World::empty();
    preserved_replacement.spawn_node(NodeKind::Camera);
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        metadata_slot(&empty, "manual-old", "Old Manual", "manual", 10),
        metadata_slot(&empty, "manual-new", "Latest Manual", "manual", 50),
    ])
    .expect("archive should validate selected capture fixture slots");

    let preview = archive
        .preview_capture_world_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
            &replacement,
            RuntimeSessionMetadata::default()
                .with_display_name("Captured Override")
                .with_tag(" captured ")
                .with_updated_at_unix_millis(80),
        )
        .expect("selected world capture preview should resolve latest manual slot");
    assert_eq!(preview.slot_id, "manual-new");
    assert!(preview.will_replace_existing);
    assert_eq!(preview.entity_count, 1);
    assert_eq!(
        preview.metadata.display_name.as_deref(),
        Some("Captured Override")
    );
    assert_eq!(preview.metadata.tags, vec!["captured"]);
    assert_eq!(
        archive
            .slot("manual-new")
            .expect("latest manual slot should remain unchanged after preview")
            .metadata
            .display_name
            .as_deref(),
        Some("Latest Manual")
    );

    archive
        .capture_world_selected_slot(
            RuntimeSessionSlotSelector::latest_updated_with_tag("manual"),
            &replacement,
            RuntimeSessionMetadata::default()
                .with_display_name("Captured Override")
                .with_tag("captured")
                .with_updated_at_unix_millis(80),
        )
        .expect("selected world capture should update resolved latest slot");
    let manifest = archive
        .manifest()
        .expect("archive should project manifest after selected capture");
    let captured = manifest
        .slot("manual-new")
        .expect("captured latest manual summary should exist");
    assert_eq!(captured.entity_count, 1);
    assert_eq!(
        captured.metadata.display_name.as_deref(),
        Some("Captured Override")
    );
    assert_eq!(captured.metadata.tags, vec!["captured"]);

    archive
        .capture_world_selected_slot_preserving_metadata(
            RuntimeSessionSlotSelector::oldest_updated_with_tag("manual"),
            &preserved_replacement,
        )
        .expect("selected preserve-metadata capture should update oldest manual slot");
    let preserved = archive
        .manifest()
        .expect("archive should project manifest after preserve capture")
        .slot("manual-old")
        .expect("oldest manual summary should exist")
        .clone();
    assert_eq!(preserved.entity_count, 1);
    assert_eq!(
        preserved.metadata.display_name.as_deref(),
        Some("Old Manual")
    );
    assert_eq!(preserved.metadata.tags, vec!["manual"]);
    assert_eq!(preserved.metadata.updated_at_unix_millis, Some(10));
}

#[test]
fn runtime_session_archive_selected_capture_to_path_previews_and_prunes_atomically() {
    let source = World::empty();
    let mut captured_world = World::empty();
    captured_world.spawn_node(NodeKind::Mesh);
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "manual-mid", "manual", 20),
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate selected path capture fixture slots");
    let root = unique_temp_root("runtime_session_selected_capture_path");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before selected path capture");
    let original_payload = fs::read_to_string(&path).expect("archive payload should be readable");

    let metadata = RuntimeSessionMetadata::default()
        .with_display_name("Captured Manual")
        .with_tag("manual")
        .with_updated_at_unix_millis(90);
    let preview =
        RuntimeSessionArchive::preview_capture_world_selected_slot_with_tag_retention_to_path(
            &path,
            " manual ",
            RuntimeSessionSlotSelector::slot_id(" manual-mid "),
            &captured_world,
            metadata.clone(),
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
        )
        .expect("selected path capture preview should resolve slot and prune clone only");
    assert_eq!(preview.capture.slot_id, "manual-mid");
    assert_eq!(preview.capture.entity_count, 1);
    assert_eq!(
        preview.prune.removed_slot_ids,
        vec!["manual-new", "manual-old"]
    );
    assert_eq!(
        preview.manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid"]
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain after preview"),
        original_payload
    );

    let report =
        RuntimeSessionArchive::capture_world_selected_slot_with_tag_retention_to_path_atomically(
            &path,
            "manual",
            RuntimeSessionSlotSelector::slot_id("manual-mid"),
            &captured_world,
            metadata,
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
        )
        .expect("selected path capture should update archive atomically");
    assert_eq!(report, preview);

    let loaded =
        RuntimeSessionArchive::load_from_path(&path).expect("archive should reload after capture");
    assert_eq!(
        loaded.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-mid"]
    );
    let selected = loaded
        .manifest()
        .expect("loaded archive should project manifest")
        .slot("manual-mid")
        .expect("selected slot summary should exist")
        .clone();
    assert_eq!(selected.entity_count, 1);
    assert_eq!(
        selected.metadata.display_name.as_deref(),
        Some("Captured Manual")
    );
    assert_eq!(selected.metadata.tags, vec!["manual"]);
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

fn metadata_slot(
    source: &World,
    slot_id: &str,
    display_name: &str,
    tag: &str,
    updated_at_unix_millis: u64,
) -> RuntimeSessionSlot {
    RuntimeSessionSlot::from_world_with_metadata(
        slot_id,
        source,
        RuntimeSessionMetadata::default()
            .with_display_name(display_name)
            .with_tag(tag)
            .with_updated_at_unix_millis(updated_at_unix_millis),
    )
    .expect("metadata slot should capture")
}
