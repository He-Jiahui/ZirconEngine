use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata,
    RuntimeSessionSlot, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_updated_slot_queries_use_secondary_indexes_without_sealing() {
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&World::empty(), "manual-old", "manual", 10),
        tagged_slot(&World::empty(), "manual-new", "manual", 50),
        tagged_slot(&World::empty(), "autosave", "autosave", 30),
    ])
    .expect("archive should validate");
    let before = archive.artifact_diagnostics();

    assert_eq!(
        archive
            .latest_updated_slot_id()
            .expect("latest selection should validate archive"),
        Some("manual-new".to_string())
    );
    assert_eq!(
        archive
            .oldest_updated_slot_id()
            .expect("oldest selection should validate archive"),
        Some("manual-old".to_string())
    );
    assert_eq!(
        archive
            .latest_updated_slot_id_with_tag(" manual ")
            .expect("tagged latest selection should validate archive"),
        Some("manual-new".to_string())
    );
    assert_eq!(
        archive
            .oldest_updated_slot_id_with_tag("manual")
            .expect("tagged oldest selection should validate archive"),
        Some("manual-old".to_string())
    );
    assert_eq!(
        archive
            .latest_updated_slot_id_with_tag("  ")
            .expect("empty tag selection should validate archive"),
        None
    );

    let after = archive.artifact_diagnostics();
    assert_eq!(after.serialize_count, before.serialize_count);
    assert_eq!(
        after.internal_json_roundtrip_count,
        before.internal_json_roundtrip_count
    );
}

#[test]
fn runtime_session_archive_loads_statistics_from_path() {
    let mut source = World::empty();
    source
        .spawn_node(crate::scene::NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag("manual")
                .with_updated_at_unix_millis(10),
        )
        .expect("manual slot should capture"),
        tagged_slot(&World::empty(), "empty", "autosave", 20),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_statistics");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before statistics load");

    let statistics = RuntimeSessionArchive::statistics_from_path(&path)
        .expect("statistics should load directly from archive path");

    assert_eq!(statistics.slot_count, 2);
    assert_eq!(statistics.total_entity_count, 1);
    assert_eq!(statistics.latest_updated_at_unix_millis, Some(20));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_reads_slot_summaries_directly_from_path() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Manual Save")
                .with_tag("manual")
                .with_updated_at_unix_millis(10),
        )
        .expect("manual slot should capture"),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_slot_summary");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before slot summary query");
    let original_payload =
        fs::read_to_string(&path).expect("archive payload should be readable before query");

    assert_eq!(
        RuntimeSessionArchive::slot_ids_from_path(&path)
            .expect("slot ids should load directly from archive path"),
        vec!["autosave".to_string(), "manual".to_string()]
    );
    assert!(
        RuntimeSessionArchive::contains_slot_from_path(&path, "manual")
            .expect("contains-slot query should load directly from archive path")
    );
    assert!(
        !RuntimeSessionArchive::contains_slot_from_path(&path, "missing")
            .expect("missing contains-slot query should still validate archive")
    );

    let summary = RuntimeSessionArchive::slot_summary_from_path(&path, "manual")
        .expect("slot summary should load directly from archive path")
        .expect("manual summary should exist");
    assert_eq!(summary.slot_id, "manual");
    assert_eq!(
        summary.metadata.display_name.as_deref(),
        Some("Manual Save")
    );
    assert!(
        RuntimeSessionArchive::slot_summary_from_path(&path, "missing")
            .expect("missing slot summary query should still validate archive")
            .is_none()
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after query"),
        original_payload
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_diffs_slot_from_path_without_mutating_target() {
    let mut source = World::empty();
    let saved_entity = source
        .spawn_node(crate::scene::NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    source
        .rename_node(saved_entity, "Saved Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world("manual", &source).expect("manual slot should capture"),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_diff");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path diff");

    let mut target = source.clone();
    let target_entity = target
        .spawn_node(crate::scene::NodeKind::Camera)
        .expect("test scene spawn should succeed");
    target
        .rename_node(target_entity, "Extra Camera")
        .expect("target entity should be named");

    let diff = RuntimeSessionArchive::diff_slot_from_path_with_world(&path, "manual", &target)
        .expect("slot should diff directly from archive path");

    assert!(!diff.matches);
    assert_eq!(diff.slot_entity_count, 1);
    assert_eq!(diff.target_entity_count, 2);
    assert!(target.find_node(target_entity).is_some());

    let manager = crate::scene::DefaultLevelManager::default();
    let level = manager.create_default_level();
    level.replace_world_and_reset_runtime_state(source.clone());
    let level_diff = RuntimeSessionArchive::diff_slot_from_path_with_level(&path, "manual", &level)
        .expect("slot should diff directly from archive path against level");
    assert!(level_diff.matches);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_previews_path_retention_without_saving() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 30),
        tagged_slot(&source, "autosave-new", "autosave", 20),
        tagged_slot(&source, "autosave-old", "autosave", 10),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_retention_preview");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path retention preview");
    let original_payload =
        fs::read_to_string(&path).expect("archive payload should be readable before preview");

    let preview = RuntimeSessionArchive::preview_prune_slots_from_path(
        &path,
        RuntimeSessionArchiveRetentionPolicy::keep_latest(2),
    )
    .expect("global retention should preview directly from archive path");

    assert_eq!(preview.removed_slot_ids, vec!["autosave-old"]);
    assert_eq!(preview.retained_slot_ids, vec!["autosave-new", "manual"]);
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after preview"),
        original_payload
    );

    let tagged_preview = RuntimeSessionArchive::preview_prune_slots_with_tag_from_path(
        &path,
        " autosave ",
        RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
    )
    .expect("tagged retention should preview directly from archive path");

    assert_eq!(tagged_preview.removed_slot_ids, vec!["autosave-old"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("previewed archive should reload without mutation")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave-new", "autosave-old", "manual"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_selects_updated_slots_directly_from_path() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-new", "manual", 50),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_slot_selection");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path slot selection");
    let original_payload =
        fs::read_to_string(&path).expect("archive payload should be readable before selection");

    assert_eq!(
        RuntimeSessionArchive::latest_updated_slot_id_from_path(&path)
            .expect("latest slot should load directly from archive path"),
        Some("manual-new".to_string())
    );
    assert_eq!(
        RuntimeSessionArchive::oldest_updated_slot_id_from_path(&path)
            .expect("oldest slot should load directly from archive path"),
        Some("manual-old".to_string())
    );
    assert_eq!(
        RuntimeSessionArchive::latest_updated_slot_id_with_tag_from_path(&path, " manual ")
            .expect("latest tagged slot should load directly from archive path"),
        Some("manual-new".to_string())
    );
    assert_eq!(
        RuntimeSessionArchive::oldest_updated_slot_id_with_tag_from_path(&path, "manual")
            .expect("oldest tagged slot should load directly from archive path"),
        Some("manual-old".to_string())
    );
    assert_eq!(
        RuntimeSessionArchive::latest_updated_slot_id_with_tag_from_path(&path, "")
            .expect("empty tag lookup should still validate archive"),
        None
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after selection"),
        original_payload
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_filters_manifest_summaries_directly_from_path() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-a",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Chapter One Manual Save")
                .with_tag(" manual ")
                .with_updated_at_unix_millis(10),
        )
        .expect("manual-a should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-b",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Chapter Two Manual Save")
                .with_tag("manual")
                .with_updated_at_unix_millis(20),
        )
        .expect("manual-b should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "autosave",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Background Autosave")
                .with_tag("autosave")
                .with_updated_at_unix_millis(30),
        )
        .expect("autosave should capture"),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_manifest_filter");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before manifest filter");
    let original_payload =
        fs::read_to_string(&path).expect("archive payload should be readable before filter");

    let manual_slots = RuntimeSessionArchive::slots_with_tag_from_path(&path, " manual ")
        .expect("tag filter should load directly from archive path");
    assert_eq!(
        manual_slots
            .iter()
            .map(|slot| slot.slot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["manual-a", "manual-b"]
    );

    let chapter_two_slots =
        RuntimeSessionArchive::slots_matching_display_name_from_path(&path, "Two")
            .expect("display-name filter should load directly from archive path");
    assert_eq!(
        chapter_two_slots
            .iter()
            .map(|slot| slot.slot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["manual-b"]
    );
    assert!(
        RuntimeSessionArchive::slots_matching_display_name_from_path(&path, "")
            .expect("empty display-name query should still validate archive")
            .is_empty()
    );
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after filter"),
        original_payload
    );

    let _ = fs::remove_dir_all(root);
}
