use super::*;

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
