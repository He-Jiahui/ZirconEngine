use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchiveMergePolicy, RuntimeSessionMetadata, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_renames_slot_at_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_rename");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path rename");

    let manifest =
        RuntimeSessionArchive::rename_slot_at_path_atomically(&path, "manual-old", " manual-new ")
            .expect("slot should rename directly on archive path");

    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-new"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("renamed archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave", "manual-new"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_updates_slot_metadata_at_path_atomically() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_update_metadata");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before metadata update");

    let manifest = RuntimeSessionArchive::update_slot_metadata_at_path_atomically(
        &path,
        "manual",
        RuntimeSessionMetadata::default()
            .with_display_name("Updated Save")
            .with_tag(" quicksave ")
            .with_tag("manual")
            .with_tag("manual")
            .with_updated_at_unix_millis(40),
    )
    .expect("slot metadata should update directly on archive path");

    let summary = manifest
        .slot("manual")
        .expect("manual summary should exist");
    assert_eq!(
        summary.metadata.display_name.as_deref(),
        Some("Updated Save")
    );
    assert_eq!(summary.metadata.tags, vec!["manual", "quicksave"]);
    assert_eq!(summary.metadata.updated_at_unix_millis, Some(40));
    let loaded = RuntimeSessionArchive::load_from_path(&path)
        .expect("metadata-updated archive should reload");
    assert_eq!(
        loaded
            .slot("manual")
            .expect("manual slot should exist")
            .metadata
            .tags,
        vec!["manual", "quicksave"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_touches_slot_at_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 50),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_touch");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before touch");

    let manifest = RuntimeSessionArchive::touch_slot_at_path_atomically(&path, "manual", 90)
        .expect("slot timestamp should update directly on archive path");

    assert_eq!(
        manifest
            .latest_updated_slot()
            .map(|summary| summary.slot_id.as_str()),
        Some("manual")
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("touched archive should reload")
            .latest_updated_slot_id()
            .expect("latest lookup should validate"),
        Some("manual".to_string())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_removes_slot_at_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_remove");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before remove");

    let manifest = RuntimeSessionArchive::remove_slot_at_path_atomically(&path, "autosave")
        .expect("slot should be removed directly from archive path");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    let missing = RuntimeSessionArchive::remove_slot_at_path_atomically(&path, "autosave");
    assert!(matches!(
        missing,
        Err(crate::scene::RuntimeSessionArchiveError::MissingSlot { .. })
    ));
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should remain readable after missing remove")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["manual"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_copies_slot_at_path_atomically() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_copy");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before copy");

    let manifest = RuntimeSessionArchive::copy_slot_with_metadata_at_path_atomically(
        &path,
        "manual",
        " manual-copy ",
        RuntimeSessionMetadata::default()
            .with_display_name("Manual Copy")
            .with_tag(" copied ")
            .with_updated_at_unix_millis(40),
    )
    .expect("slot should copy directly on archive path");

    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["manual", "manual-copy"]
    );
    assert_eq!(
        manifest
            .slot("manual-copy")
            .expect("copy summary should exist")
            .metadata
            .tags,
        vec!["copied"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("copied archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["manual", "manual-copy"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_imports_single_slot_at_path_atomically() {
    let source = World::empty();
    let target =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("target archive should validate");
    let incoming =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "incoming", "import", 30)])
            .expect("incoming archive should validate");
    let root = unique_temp_root("runtime_session_path_single_slot_import");
    let path = root.join("sessions").join("archive.zrsession.json");
    target
        .save_to_path_atomically(&path)
        .expect("target archive should save before single-slot import");

    let manifest =
        RuntimeSessionArchive::import_slot_from_archive_with_metadata_at_path_atomically(
            &path,
            &incoming,
            "incoming",
            " imported-copy ",
            RuntimeSessionMetadata::default()
                .with_display_name("Imported Copy")
                .with_tag(" imported ")
                .with_updated_at_unix_millis(70),
        )
        .expect("single incoming slot should import into archive path");

    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["imported-copy", "manual"]
    );
    let imported = manifest
        .slot("imported-copy")
        .expect("imported summary should exist");
    assert_eq!(
        imported.metadata.display_name.as_deref(),
        Some("Imported Copy")
    );
    assert_eq!(imported.metadata.tags, vec!["imported"]);
    assert_eq!(imported.metadata.updated_at_unix_millis, Some(70));
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("single-slot imported archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["imported-copy", "manual"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_imports_single_slot_from_path_at_path_atomically() {
    let source = World::empty();
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-source", "manual", 10),
        tagged_slot(&source, "incoming", "incoming", 30),
    ])
    .expect("incoming archive should validate");
    let target = RuntimeSessionArchive::from_slots(vec![tagged_slot(
        &source,
        "manual-target",
        "manual",
        50,
    )])
    .expect("target archive should validate");
    let root = unique_temp_root("runtime_session_path_to_path_single_slot_import");
    let source_path = root.join("source").join("archive.zrsession.json");
    let target_path = root.join("target").join("archive.zrsession.json");
    incoming
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before path-to-path import");
    target
        .save_to_path_atomically(&target_path)
        .expect("target archive should save before path-to-path import");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");

    let manifest =
        RuntimeSessionArchive::import_slot_from_archive_path_with_metadata_at_path_atomically(
            &target_path,
            &source_path,
            "incoming",
            " imported-from-path ",
            RuntimeSessionMetadata::default()
                .with_display_name("Imported From Path")
                .with_tag(" path-import ")
                .with_updated_at_unix_millis(90),
        )
        .expect("single source path slot should import into target archive path");

    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["imported-from-path", "manual-target"]
    );
    let imported = manifest
        .slot("imported-from-path")
        .expect("imported path summary should exist");
    assert_eq!(
        imported.metadata.display_name.as_deref(),
        Some("Imported From Path")
    );
    assert_eq!(imported.metadata.tags, vec!["path-import"]);
    assert_eq!(imported.metadata.updated_at_unix_millis, Some(90));
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("target archive should reload after path-to-path import")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["imported-from-path", "manual-target"]
    );
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after import"),
        source_payload
    );
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_saves_single_slot_archive_from_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 30),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_single_slot_archive");
    let source_path = root.join("sessions").join("archive.zrsession.json");
    let target_path = root.join("exports").join("autosave.zrsession.json");
    archive
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before single-slot archive export");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");

    let manifest = RuntimeSessionArchive::save_single_slot_archive_from_path_atomically(
        &source_path,
        "autosave",
        &target_path,
    )
    .expect("one source slot should save as a standalone archive");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["autosave"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("standalone slot archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave"]
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&target_path)
            .expect("standalone slot archive should reload for metadata")
            .slot("autosave")
            .expect("autosave slot should exist")
            .metadata
            .tags,
        vec!["autosave"]
    );
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after export"),
        source_payload
    );
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_saves_single_slot_archive_from_memory_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "quicksave", "quicksave", 50),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_memory_single_slot_archive");
    let target_path = root.join("exports").join("manual.zrsession.json");

    let manifest = archive
        .save_single_slot_archive_to_path_atomically("manual", &target_path)
        .expect("one in-memory slot should save as a standalone archive");

    assert_eq!(manifest.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    let exported = RuntimeSessionArchive::load_from_path(&target_path)
        .expect("standalone in-memory slot archive should reload");
    assert_eq!(exported.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    assert_eq!(
        exported
            .slot("manual")
            .expect("manual slot should exist")
            .metadata
            .tags,
        vec!["manual"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["manual", "quicksave"]
    );
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_merges_archive_at_path_atomically() {
    let source = World::empty();
    let target =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("target archive should validate");
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "replacement", 50),
        tagged_slot(&source, "imported", "import", 20),
    ])
    .expect("incoming archive should validate");
    let root = unique_temp_root("runtime_session_path_merge");
    let path = root.join("sessions").join("archive.zrsession.json");
    target
        .save_to_path_atomically(&path)
        .expect("target archive should save before merge");

    let report = RuntimeSessionArchive::merge_archive_at_path_atomically(
        &path,
        &incoming,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect("incoming archive should merge into archive path");

    assert_eq!(report.inserted_slot_ids, vec!["imported"]);
    assert_eq!(report.replaced_slot_ids, vec!["manual"]);
    let loaded =
        RuntimeSessionArchive::load_from_path(&path).expect("merged archive should reload");
    assert_eq!(
        loaded.slot_ids().collect::<Vec<_>>(),
        vec!["imported", "manual"]
    );
    assert_eq!(
        loaded
            .latest_updated_slot_id_with_tag("replacement")
            .expect("replacement tag selection should validate"),
        Some("manual".to_string())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_merges_archive_from_path_at_path_atomically() {
    let source = World::empty();
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "incoming-replacement", 80),
        tagged_slot(&source, "bonus", "incoming", 30),
    ])
    .expect("incoming archive should validate");
    let target = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "target", 10),
        tagged_slot(&source, "autosave", "target", 20),
    ])
    .expect("target archive should validate");
    let root = unique_temp_root("runtime_session_path_to_path_merge");
    let source_path = root.join("source").join("archive.zrsession.json");
    let target_path = root.join("target").join("archive.zrsession.json");
    incoming
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before path-to-path merge");
    target
        .save_to_path_atomically(&target_path)
        .expect("target archive should save before path-to-path merge");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");

    let report = RuntimeSessionArchive::merge_archive_from_path_at_path_atomically(
        &target_path,
        &source_path,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect("source archive path should merge into target archive path");

    assert_eq!(report.inserted_slot_ids, vec!["bonus"]);
    assert_eq!(report.replaced_slot_ids, vec!["manual"]);
    let loaded = RuntimeSessionArchive::load_from_path(&target_path)
        .expect("target archive should reload after path-to-path merge");
    assert_eq!(
        loaded.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "bonus", "manual"]
    );
    assert_eq!(
        loaded
            .latest_updated_slot_id_with_tag("incoming-replacement")
            .expect("replacement tag selection should validate"),
        Some("manual".to_string())
    );
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after path-to-path merge"),
        source_payload
    );
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_previews_merge_without_mutating_archives() {
    let source = World::empty();
    let target = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "target", 10),
        tagged_slot(&source, "autosave", "target", 20),
    ])
    .expect("target archive should validate");
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "incoming", 80),
        tagged_slot(&source, "bonus", "incoming", 30),
    ])
    .expect("incoming archive should validate");

    let report = target
        .preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)
        .expect("merge preview should report keep-existing result");

    assert_eq!(report.inserted_slot_ids, vec!["bonus"]);
    assert_eq!(report.skipped_slot_ids, vec!["manual"]);
    assert!(report.replaced_slot_ids.is_empty());
    assert_eq!(
        target.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    assert_eq!(
        incoming.slot_ids().collect::<Vec<_>>(),
        vec!["bonus", "manual"]
    );
}

#[test]
fn runtime_session_archive_previews_merge_from_path_without_mutating_archives() {
    let source = World::empty();
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "incoming-replacement", 80),
        tagged_slot(&source, "bonus", "incoming", 30),
    ])
    .expect("incoming archive should validate");
    let target =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "target", 10)])
            .expect("target archive should validate");
    let root = unique_temp_root("runtime_session_path_to_path_merge_preview");
    let source_path = root.join("source").join("archive.zrsession.json");
    let target_path = root.join("target").join("archive.zrsession.json");
    incoming
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before path-to-path merge preview");
    target
        .save_to_path_atomically(&target_path)
        .expect("target archive should save before path-to-path merge preview");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");
    let target_payload =
        fs::read_to_string(&target_path).expect("target archive payload should be readable");

    let report = RuntimeSessionArchive::preview_merge_archive_from_path_at_path(
        &target_path,
        &source_path,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect("source archive path merge preview should report replace-existing result");

    assert_eq!(report.inserted_slot_ids, vec!["bonus"]);
    assert_eq!(report.replaced_slot_ids, vec!["manual"]);
    assert!(report.skipped_slot_ids.is_empty());
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after merge preview"),
        source_payload
    );
    assert_eq!(
        fs::read_to_string(&target_path)
            .expect("target archive payload should remain readable after merge preview"),
        target_payload
    );
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}
