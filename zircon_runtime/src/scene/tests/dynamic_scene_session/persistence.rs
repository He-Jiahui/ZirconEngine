use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchivePathStatus, RuntimeSessionArchiveRetentionPolicy,
    World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_saves_atomically_without_temp_leftovers() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "autosave", "autosave", 10)])
            .expect("archive should accept slot");
    let root = unique_temp_root("runtime_session_atomic_save");
    let path = root.join("sessions").join("archive.zrsession.json");

    archive
        .save_to_path_atomically(&path)
        .expect("archive should save through a same-directory temporary file");

    let loaded =
        RuntimeSessionArchive::load_from_path(&path).expect("atomic save output should reload");
    assert_eq!(
        loaded
            .latest_updated_slot_id_with_tag("autosave")
            .expect("latest autosave lookup should validate"),
        Some("autosave".to_string())
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_atomic_save_replaces_existing_target_without_leftovers() {
    let source = World::empty();
    let original =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 1)])
            .expect("archive should accept original slot");
    let replacement =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "autosave", "autosave", 20)])
            .expect("archive should accept replacement slot");
    let root = unique_temp_root("runtime_session_atomic_replace");
    let path = root.join("sessions").join("archive.zrsession.json");

    original
        .save_to_path(&path)
        .expect("original archive should save directly");
    replacement
        .save_to_path_atomically(&path)
        .expect("atomic save should replace existing archive");

    let loaded =
        RuntimeSessionArchive::load_from_path(&path).expect("replaced archive should reload");
    assert_eq!(loaded.slot_ids().collect::<Vec<_>>(), vec!["autosave"]);
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_preview_save_to_path_reports_targets_without_writing_files() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should accept preview slot");
    let root = unique_temp_root("runtime_session_full_save_preview");
    let missing_path = root.join("missing").join("archive.zrsession.json");

    let missing_target = archive
        .preview_save_to_path(&missing_path)
        .expect("missing-target full-save preview should validate");
    assert_eq!(missing_target.target_path, missing_path);
    assert!(!missing_target.will_replace_target);
    assert_eq!(missing_target.statistics.slot_count, 1);
    assert_eq!(
        missing_target.statistics.latest_updated_at_unix_millis,
        Some(10)
    );
    assert!(!missing_path.exists());
    assert!(!missing_path
        .parent()
        .expect("missing path should have parent")
        .exists());

    let existing_path = root.join("sessions").join("archive.zrsession.json");
    fs::create_dir_all(
        existing_path
            .parent()
            .expect("existing path should have parent"),
    )
    .expect("existing target parent should be created");
    fs::write(&existing_path, "existing payload").expect("existing target fixture should write");
    let existing_payload =
        fs::read_to_string(&existing_path).expect("existing target payload should be readable");
    let existing_target = archive
        .preview_save_to_path(&existing_path)
        .expect("existing-file full-save preview should validate");
    assert_eq!(existing_target.target_path, existing_path);
    assert!(existing_target.will_replace_target);
    assert_eq!(
        fs::read_to_string(&existing_path)
            .expect("existing target payload should remain readable after preview"),
        existing_payload
    );
    assert!(temporary_archive_leftovers(
        existing_path
            .parent()
            .expect("existing path should have parent")
    )
    .is_empty());

    let directory_target = root.join("sessions").join("directory-target");
    fs::create_dir_all(&directory_target).expect("directory target fixture should be created");
    let non_file_target = archive.preview_save_to_path(&directory_target);
    assert!(matches!(
        non_file_target,
        Err(crate::scene::RuntimeSessionArchiveError::Io(error))
            if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert!(directory_target.is_dir());
    assert!(temporary_archive_leftovers(
        existing_path
            .parent()
            .expect("existing path should have parent")
    )
    .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_preview_save_to_path_rejects_parent_file_without_writes() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should accept parent-file preview slot");
    let root = unique_temp_root("runtime_session_full_save_preview_parent_file");
    fs::create_dir_all(&root).expect("preview root should be created");
    let parent_file = root.join("parent-file");
    fs::write(&parent_file, "parent payload").expect("parent file fixture should write");
    let target_path = parent_file.join("archive.zrsession.json");

    let parent_file_target = archive.preview_save_to_path(&target_path);

    assert!(matches!(
        parent_file_target,
        Err(crate::scene::RuntimeSessionArchiveError::Io(error))
            if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert_eq!(
        fs::read_to_string(&parent_file)
            .expect("parent file payload should remain readable after preview"),
        "parent payload"
    );
    assert!(parent_file.is_file());
    assert!(!target_path.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_loads_manifest_from_path_without_manual_archive_projection() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should accept manifest slots");
    let root = unique_temp_root("runtime_session_manifest_from_path");
    let path = root.join("sessions").join("archive.zrsession.json");

    archive
        .save_to_path_atomically(&path)
        .expect("archive should save atomically before manifest read");

    let manifest = RuntimeSessionArchive::load_manifest_from_path(&path)
        .expect("manifest should load from archive path");
    assert_eq!(
        manifest
            .slots_with_tag("manual")
            .into_iter()
            .map(|summary| summary.slot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["manual"]
    );
    assert_eq!(
        manifest
            .latest_updated_slot_with_tag("autosave")
            .map(|summary| summary.slot_id.as_str()),
        Some("autosave")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_inspects_path_status_without_treating_missing_as_invalid() {
    let source = World::empty();
    let archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "manual", "manual", 10)])
            .expect("archive should accept inspect slot");
    let root = unique_temp_root("runtime_session_path_status");
    let path = root.join("sessions").join("archive.zrsession.json");
    let invalid_path = root.join("sessions").join("invalid.zrsession.json");

    assert!(RuntimeSessionArchive::inspect_path(&path).is_missing());

    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before inspect");
    match RuntimeSessionArchive::inspect_path(&path) {
        RuntimeSessionArchivePathStatus::Available { manifest } => {
            assert_eq!(
                manifest
                    .slots
                    .iter()
                    .map(|slot| slot.slot_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["manual"]
            );
        }
        status => panic!("expected available archive status, got {status:?}"),
    }

    fs::write(&invalid_path, "{ invalid-json").expect("invalid archive fixture should write");
    match RuntimeSessionArchive::inspect_path(&invalid_path) {
        RuntimeSessionArchivePathStatus::Invalid { .. } => {}
        status => panic!("expected invalid archive status, got {status:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_prunes_slots_at_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "slot-new", "manual", 30),
        tagged_slot(&source, "slot-mid", "manual", 20),
        tagged_slot(&source, "slot-old", "manual", 10),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_prune");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before prune");

    let report = RuntimeSessionArchive::prune_slots_at_path_atomically(
        &path,
        RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
    )
    .expect("archive path should prune globally");

    assert_eq!(report.removed_slot_ids, vec!["slot-mid", "slot-old"]);
    assert_eq!(report.retained_slot_ids, vec!["slot-new"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("pruned archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["slot-new"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_session_archive_prunes_tag_bucket_at_path_atomically() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 5),
        tagged_slot(&source, "autosave-new", "autosave", 30),
        tagged_slot(&source, "autosave-old", "autosave", 10),
    ])
    .expect("archive should validate");
    let root = unique_temp_root("runtime_session_path_prune_tag");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before tag prune");

    let report = RuntimeSessionArchive::prune_slots_with_tag_at_path_atomically(
        &path,
        " autosave ",
        RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
    )
    .expect("archive path should prune one tag bucket");

    assert_eq!(report.removed_slot_ids, vec!["autosave-old"]);
    assert_eq!(report.retained_slot_ids, vec!["autosave-new", "manual"]);
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("tag-pruned archive should reload")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave-new", "manual"]
    );

    let _ = fs::remove_dir_all(root);
}
