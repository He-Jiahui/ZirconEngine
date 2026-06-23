use super::*;

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
fn runtime_session_archive_previews_slot_copy_without_mutating_archive() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should validate");

    let report = archive
        .preview_copy_slot_with_metadata(
            "manual",
            " manual-copy ",
            RuntimeSessionMetadata::default()
                .with_display_name("Manual Copy")
                .with_tag(" copied ")
                .with_tag("copied")
                .with_updated_at_unix_millis(40),
        )
        .expect("slot copy preview should validate");

    assert_eq!(report.source_slot_id, "manual");
    assert_eq!(report.destination_slot_id, "manual-copy");
    assert_eq!(report.metadata.display_name.as_deref(), Some("Manual Copy"));
    assert_eq!(report.metadata.tags, vec!["copied"]);
    assert_eq!(report.metadata.updated_at_unix_millis, Some(40));
    assert_eq!(report.entity_count, 0);
    assert_eq!(report.resource_count, 0);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["manual"]);

    let duplicate = archive.preview_copy_slot("manual", " manual ");
    assert!(matches!(
        duplicate,
        Err(crate::scene::RuntimeSessionArchiveError::DuplicateSlotId { .. })
    ));
}

#[test]
fn runtime_session_archive_previews_slot_copy_from_path_without_mutating_archive() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_copy_preview");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before copy preview");
    let payload = fs::read_to_string(&path).expect("archive payload should be readable");

    let report = RuntimeSessionArchive::preview_copy_slot_with_metadata_from_path(
        &path,
        "manual",
        " manual-copy ",
        RuntimeSessionMetadata::default()
            .with_display_name("Manual Copy")
            .with_tag(" copied ")
            .with_updated_at_unix_millis(40),
    )
    .expect("slot copy path preview should validate");

    assert_eq!(report.source_slot_id, "manual");
    assert_eq!(report.destination_slot_id, "manual-copy");
    assert_eq!(report.metadata.tags, vec!["copied"]);
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after preview"),
        payload
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should reload after copy preview")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["manual"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
