use std::fs;

use crate::scene::{
    NodeKind, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlotSelector,
    World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_prunes_only_matching_tag_bucket() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual-new", "manual", 40),
        tagged_slot(&source, "manual-old", "manual", 10),
        tagged_slot(&source, "manual-protected", "manual", 5),
        tagged_slot(&source, "autosave", "autosave", 1),
    ])
    .expect("archive should accept tagged slots");

    let preview = archive
        .preview_prune_slots_with_tag(
            " manual ",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(2)
                .with_protected_slot("manual-protected"),
        )
        .expect("tag bucket prune preview should validate archive");
    assert_eq!(preview.removed_slot_ids, vec!["manual-old"]);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-new", "manual-old", "manual-protected"]
    );
    let revision_before_prune = archive.revision();

    let report = archive
        .prune_slots_with_tag(
            " manual ",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(2)
                .with_protected_slot("manual-protected"),
        )
        .expect("tag bucket prune should validate archive");

    assert_eq!(report, preview);
    assert_eq!(
        report.retained_slot_ids,
        vec!["autosave", "manual-new", "manual-protected"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual-new", "manual-protected"]
    );
    assert_eq!(archive.revision(), revision_before_prune + 1);
}

#[test]
fn runtime_session_archive_prune_tag_empty_query_is_noop() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should accept tagged slots");
    let revision_before_prune = archive.revision();

    let report = archive
        .prune_slots_with_tag("", RuntimeSessionArchiveRetentionPolicy::keep_latest(0))
        .expect("empty tag prune should validate archive");

    assert!(report.is_empty());
    assert_eq!(report.retained_slot_ids, vec!["autosave", "manual"]);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    assert_eq!(archive.revision(), revision_before_prune);
}

#[test]
fn runtime_session_archive_previews_global_retention_without_mutation() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "slot-new", "manual", 30),
        tagged_slot(&source, "slot-mid", "manual", 20),
        tagged_slot(&source, "slot-old", "manual", 10),
    ])
    .expect("archive should accept slots");

    let preview = archive
        .preview_prune_slots(RuntimeSessionArchiveRetentionPolicy::keep_latest(1))
        .expect("global prune preview should validate archive");

    assert_eq!(preview.removed_slot_ids, vec!["slot-mid", "slot-old"]);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["slot-mid", "slot-new", "slot-old"]
    );
    let revision_before_prune = archive.revision();

    let report = archive
        .prune_slots(RuntimeSessionArchiveRetentionPolicy::keep_latest(1))
        .expect("global prune should validate archive");

    assert_eq!(report, preview);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["slot-new"]);
    assert_eq!(archive.revision(), revision_before_prune + 1);
}

#[test]
fn runtime_session_archive_prune_plan_rejects_stale_commit_without_removing_rows() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "slot-new", "manual", 30),
        tagged_slot(&source, "slot-old", "manual", 10),
    ])
    .expect("archive should accept slots");
    let plan = archive
        .prepare_prune_slots(RuntimeSessionArchiveRetentionPolicy::keep_latest(1))
        .expect("prune plan should validate archive");

    archive
        .touch_slot("slot-old", 40)
        .expect("intervening mutation should publish a new revision");
    let error = plan
        .commit(&mut archive)
        .expect_err("stale prune plan must not publish its prior deletions");

    assert!(matches!(
        error,
        RuntimeSessionArchiveError::StalePrunePlan { .. }
    ));
    assert_eq!(archive.slot_count(), 2);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["slot-new", "slot-old"],
        "stale pruning must leave every dense row intact"
    );
}

#[test]
fn runtime_session_archive_preview_capture_retention_projects_without_mutating_archive() {
    let empty = World::empty();
    let mut captured_world = World::empty();
    captured_world
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&empty, "autosave-old", "autosave", 50),
        tagged_slot(&empty, "manual-new", "manual", 40),
        tagged_slot(&empty, "manual-old", "manual", 10),
    ])
    .expect("archive should accept capture-retention preview slots");

    let preview = archive
        .preview_capture_world_slot_with_tag_retention(
            " autosave ",
            " autosave-new ",
            &captured_world,
            RuntimeSessionMetadata::default()
                .with_tag(" autosave ")
                .with_updated_at_unix_millis(1),
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
        )
        .expect("capture-retention preview should project the staged archive only");

    assert_eq!(preview.capture.slot_id, "autosave-new");
    assert_eq!(preview.capture.entity_count, 1);
    assert_eq!(preview.prune.removed_slot_ids, vec!["autosave-old"]);
    assert_eq!(
        preview.manifest.slot_ids().collect::<Vec<_>>(),
        vec!["autosave-new", "manual-new", "manual-old"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave-old", "manual-new", "manual-old"]
    );
    assert!(!archive.contains_slot("autosave-new"));
}

#[test]
fn runtime_session_archive_capture_empty_tag_retention_does_not_prune() {
    let source = World::empty();
    let mut archive =
        RuntimeSessionArchive::from_slots(vec![tagged_slot(&source, "slot-new", "manual", 50)])
            .expect("archive should accept empty-tag capture-retention slots");
    let revision_before_capture = archive.revision();

    let report = archive
        .capture_world_slot_with_tag_retention(
            "   ",
            " quicksave ",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag(" quicksave ")
                .with_updated_at_unix_millis(1),
            RuntimeSessionArchiveRetentionPolicy::keep_latest(0),
        )
        .expect("empty tag capture retention should preserve the full archive");

    assert!(report.prune.removed_slot_ids.is_empty());
    assert_eq!(
        report.manifest.slot_ids().collect::<Vec<_>>(),
        vec!["quicksave", "slot-new"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["quicksave", "slot-new"]
    );
    assert_eq!(archive.revision(), revision_before_capture + 1);
}

#[test]
fn runtime_session_archive_capture_retention_protects_captured_slot_before_pruning() {
    let empty = World::empty();
    let mut captured_world = World::empty();
    captured_world
        .spawn_node(NodeKind::Camera)
        .expect("test scene spawn should succeed");
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&empty, "slot-new", "manual", 50),
        tagged_slot(&empty, "slot-old", "manual", 10),
    ])
    .expect("archive should accept capture-retention commit slots");
    let revision_before_capture = archive.revision();

    let report = archive
        .capture_world_slot_with_retention(
            " quicksave ",
            &captured_world,
            RuntimeSessionMetadata::default()
                .with_tag(" quicksave ")
                .with_updated_at_unix_millis(1),
            RuntimeSessionArchiveRetentionPolicy::keep_latest(0),
        )
        .expect("capture-retention commit should protect the captured slot");

    assert_eq!(report.capture.slot_id, "quicksave");
    assert_eq!(report.capture.entity_count, 1);
    assert_eq!(report.prune.removed_slot_ids, vec!["slot-new", "slot-old"]);
    assert_eq!(
        report.manifest.slot_ids().collect::<Vec<_>>(),
        vec!["quicksave"]
    );
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["quicksave"]);
    assert_eq!(archive.revision(), revision_before_capture + 1);
    assert_eq!(
        archive
            .slot("quicksave")
            .expect("captured slot should survive retention")
            .metadata
            .tags,
        vec!["quicksave"]
    );
}

#[test]
fn runtime_session_archive_selected_retention_protects_latest_tagged_slot() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "autosave-new", "autosave", 60),
        tagged_slot(&source, "manual-new", "manual", 40),
        tagged_slot(&source, "manual-old", "manual", 10),
    ])
    .expect("archive should accept selected-retention slots");

    let preview = archive
        .preview_prune_slots_with_selected_protection(
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
        )
        .expect("selected retention preview should protect latest tagged slot");

    assert_eq!(preview.removed_slot_ids, vec!["autosave-new", "manual-old"]);
    assert_eq!(preview.retained_slot_ids, vec!["manual-new"]);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave-new", "manual-new", "manual-old"]
    );

    let report = archive
        .prune_slots_with_selected_protection(
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::latest_updated_with_tag(" manual "),
        )
        .expect("selected retention commit should protect latest tagged slot");

    assert_eq!(report, preview);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["manual-new"]);
}

#[test]
fn runtime_session_archive_tag_selected_retention_ignores_protection_outside_bucket() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "autosave-new", "autosave", 50),
        tagged_slot(&source, "autosave-old", "autosave", 10),
        tagged_slot(&source, "manual-protected", "manual", 1),
    ])
    .expect("archive should accept tag selected-retention slots");

    let preview = archive
        .preview_prune_slots_with_tag_and_selected_protection(
            " autosave ",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::slot_id(" manual-protected "),
        )
        .expect("tag selected retention preview should ignore protection outside bucket");

    assert_eq!(preview.removed_slot_ids, vec!["autosave-old"]);
    assert_eq!(
        preview.retained_slot_ids,
        vec!["autosave-new", "manual-protected"]
    );
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave-new", "autosave-old", "manual-protected"]
    );

    let report = archive
        .prune_slots_with_tag_and_selected_protection(
            " autosave ",
            RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
            RuntimeSessionSlotSelector::slot_id(" manual-protected "),
        )
        .expect("tag selected retention commit should preserve tag scope");

    assert_eq!(report, preview);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave-new", "manual-protected"]
    );
}

#[test]
fn runtime_session_archive_path_selected_retention_preview_does_not_write_archive() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "autosave-new", "autosave", 60),
        tagged_slot(&source, "manual-new", "manual", 40),
        tagged_slot(&source, "manual-old", "manual", 10),
    ])
    .expect("archive should accept path selected-retention slots");
    let root = unique_temp_root("runtime_session_path_selected_retention_preview");
    let path = root.join("sessions").join("archive.zrsession.json");
    archive
        .save_to_path_atomically(&path)
        .expect("archive should save before path selected retention preview");
    let original_payload =
        fs::read_to_string(&path).expect("archive payload should be readable before preview");

    let preview = RuntimeSessionArchive::preview_prune_slots_with_selected_protection_from_path(
        &path,
        RuntimeSessionArchiveRetentionPolicy::keep_latest(1),
        RuntimeSessionSlotSelector::oldest_updated_with_tag(" manual "),
    )
    .expect("selected retention should preview directly from archive path");

    assert_eq!(preview.removed_slot_ids, vec!["autosave-new", "manual-new"]);
    assert_eq!(preview.retained_slot_ids, vec!["manual-old"]);
    assert_eq!(
        fs::read_to_string(&path).expect("archive payload should remain readable after preview"),
        original_payload
    );
    assert_eq!(
        RuntimeSessionArchive::load_from_path(&path)
            .expect("previewed archive should reload without mutation")
            .slot_ids()
            .collect::<Vec<_>>(),
        vec!["autosave-new", "manual-new", "manual-old"]
    );
    assert!(
        temporary_archive_leftovers(path.parent().expect("session path should have parent"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}
