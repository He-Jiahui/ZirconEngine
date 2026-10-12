use super::*;

#[test]
fn runtime_session_archive_previews_slot_mutations_without_mutating_archive() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate");

    let rename = archive
        .preview_rename_slot("manual", " manual-renamed ")
        .expect("rename preview should validate");
    assert_eq!(rename.source_slot_id, "manual");
    assert_eq!(
        rename.destination_slot_id.as_deref(),
        Some("manual-renamed")
    );
    assert_eq!(rename.metadata.tags, vec!["manual"]);
    assert_eq!(rename.metadata.updated_at_unix_millis, Some(10));
    assert_eq!(rename.entity_count, 0);
    assert_eq!(rename.resource_count, 0);

    let metadata = archive
        .preview_update_slot_metadata(
            "manual",
            RuntimeSessionMetadata::default()
                .with_display_name("Updated Manual")
                .with_tag(" updated ")
                .with_tag("updated")
                .with_updated_at_unix_millis(40),
        )
        .expect("metadata preview should validate");
    assert_eq!(metadata.source_slot_id, "manual");
    assert_eq!(metadata.destination_slot_id, None);
    assert_eq!(
        metadata.metadata.display_name.as_deref(),
        Some("Updated Manual")
    );
    assert_eq!(metadata.metadata.tags, vec!["updated"]);
    assert_eq!(metadata.metadata.updated_at_unix_millis, Some(40));

    let touch = archive
        .preview_touch_slot("manual", 90)
        .expect("touch preview should validate");
    assert_eq!(touch.source_slot_id, "manual");
    assert_eq!(touch.destination_slot_id, None);
    assert_eq!(touch.metadata.tags, vec!["manual"]);
    assert_eq!(touch.metadata.updated_at_unix_millis, Some(90));

    let remove = archive
        .preview_remove_slot("autosave")
        .expect("remove preview should validate");
    assert_eq!(remove.source_slot_id, "autosave");
    assert_eq!(remove.destination_slot_id, None);
    assert_eq!(remove.metadata.tags, vec!["autosave"]);
    assert_eq!(remove.metadata.updated_at_unix_millis, Some(20));

    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    let duplicate = archive.preview_rename_slot("manual", " autosave ");
    assert!(matches!(
        duplicate,
        Err(crate::scene::RuntimeSessionArchiveError::DuplicateSlotId { .. })
    ));
}

#[test]
fn runtime_session_archive_previews_slot_mutations_from_path_without_mutating_archive() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_mutation_preview");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before slot mutation previews");
    let payload = fs::read_to_string(&path).expect("archive payload should be readable");

    let rename =
        RuntimeSessionArchive::preview_rename_slot_from_path(&path, "manual", " manual-renamed ")
            .expect("path rename preview should validate");
    assert_eq!(rename.source_slot_id, "manual");
    assert_eq!(
        rename.destination_slot_id.as_deref(),
        Some("manual-renamed")
    );

    let metadata = RuntimeSessionArchive::preview_update_slot_metadata_from_path(
        &path,
        "manual",
        RuntimeSessionMetadata::default()
            .with_display_name("Updated Manual")
            .with_tag(" updated ")
            .with_updated_at_unix_millis(40),
    )
    .expect("path metadata preview should validate");
    assert_eq!(metadata.source_slot_id, "manual");
    assert_eq!(metadata.metadata.tags, vec!["updated"]);
    assert_eq!(metadata.metadata.updated_at_unix_millis, Some(40));

    let touch = RuntimeSessionArchive::preview_touch_slot_from_path(&path, "manual", 90)
        .expect("path touch preview should validate");
    assert_eq!(touch.source_slot_id, "manual");
    assert_eq!(touch.metadata.updated_at_unix_millis, Some(90));

    let remove = RuntimeSessionArchive::preview_remove_slot_from_path(&path, "autosave")
        .expect("path remove preview should validate");
    assert_eq!(remove.source_slot_id, "autosave");
    assert_eq!(remove.metadata.tags, vec!["autosave"]);

    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after previews"),
        payload
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("archive should reload after slot mutation previews")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
