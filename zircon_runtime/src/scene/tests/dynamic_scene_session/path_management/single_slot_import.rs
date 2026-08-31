use super::*;

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
    assert!(
        temporary_archive_leftovers(
            source_path
                .parent()
                .expect("source path should have parent")
        )
        .is_empty()
    );
    assert!(
        temporary_archive_leftovers(
            target_path
                .parent()
                .expect("target path should have parent")
        )
        .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_previews_single_slot_import_without_mutating_archives() {
    let source = World::empty();
    let target =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "target", 10)])
            .expect("target archive should validate");
    let incoming =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "incoming", "import", 30)])
            .expect("incoming archive should validate");

    let report = target
        .preview_import_slot_from_archive_with_metadata(
            &incoming,
            "incoming",
            " imported-preview ",
            RuntimeSessionMetadata::default()
                .with_display_name("Imported Preview")
                .with_tag(" preview ")
                .with_tag("preview")
                .with_updated_at_unix_millis(70),
        )
        .expect("single-slot import preview should validate");

    assert_eq!(report.source_slot_id, "incoming");
    assert_eq!(report.destination_slot_id, "imported-preview");
    assert_eq!(
        report.metadata.display_name.as_deref(),
        Some("Imported Preview")
    );
    assert_eq!(report.metadata.tags, vec!["preview"]);
    assert_eq!(report.metadata.updated_at_unix_millis, Some(70));
    assert_eq!(report.entity_count, 0);
    assert_eq!(report.resource_count, 0);
    assert_eq!(target.slot_ids().collect::<Vec<_>>(), vec!["manual"]);
    assert_eq!(incoming.slot_ids().collect::<Vec<_>>(), vec!["incoming"]);

    let duplicate = target.preview_import_slot_from_archive(&incoming, "incoming", " manual ");
    assert!(matches!(
        duplicate,
        Err(crate::scene::RuntimeSessionArchiveError::DuplicateSlotId { .. })
    ));
}

#[test]
fn runtime_session_archive_previews_single_slot_import_from_path_without_mutating_archives() {
    let source = World::empty();
    let incoming = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "incoming", "incoming", 30),
        tagged_slot(&source, "other", "incoming", 40),
    ])
    .expect("incoming archive should validate");
    let target =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "target", 10)])
            .expect("target archive should validate");
    let root = unique_temp_root("runtime_session_path_single_slot_import_preview");
    let source_path = root.join("source").join("archive.zrsession.json");
    let target_path = root.join("target").join("archive.zrsession.json");
    incoming
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before import preview");
    target
        .save_to_path_atomically(&target_path)
        .expect("target archive should save before import preview");
    let source_payload =
        fs::read_to_string(&source_path).expect("source archive payload should be readable");
    let target_payload =
        fs::read_to_string(&target_path).expect("target archive payload should be readable");

    let report =
        RuntimeSessionArchive::preview_import_slot_from_archive_path_with_metadata_at_path(
            &target_path,
            &source_path,
            "incoming",
            " path-preview ",
            RuntimeSessionMetadata::default()
                .with_display_name("Path Preview")
                .with_tag(" path-preview ")
                .with_updated_at_unix_millis(90),
        )
        .expect("source path single-slot import preview should validate");

    assert_eq!(report.source_slot_id, "incoming");
    assert_eq!(report.destination_slot_id, "path-preview");
    assert_eq!(report.metadata.tags, vec!["path-preview"]);
    assert_eq!(report.metadata.updated_at_unix_millis, Some(90));
    assert_eq!(
        fs::read_to_string(&source_path)
            .expect("source archive payload should remain readable after preview"),
        source_payload
    );
    assert_eq!(
        fs::read_to_string(&target_path)
            .expect("target archive payload should remain readable after preview"),
        target_payload
    );
    assert!(
        temporary_archive_leftovers(
            source_path
                .parent()
                .expect("source path should have parent")
        )
        .is_empty()
    );
    assert!(
        temporary_archive_leftovers(
            target_path
                .parent()
                .expect("target path should have parent")
        )
        .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
