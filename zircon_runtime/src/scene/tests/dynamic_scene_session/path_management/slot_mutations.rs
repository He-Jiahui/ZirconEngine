use super::*;

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
