use crate::scene::{RuntimeSessionArchive, RuntimeSessionArchiveRetentionPolicy, World};

use super::tagged_slot;

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
}

#[test]
fn runtime_session_archive_prune_tag_empty_query_is_noop() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        tagged_slot(&source, "manual", "manual", 10),
        tagged_slot(&source, "autosave", "autosave", 20),
    ])
    .expect("archive should accept tagged slots");

    let report = archive
        .prune_slots_with_tag("", RuntimeSessionArchiveRetentionPolicy::keep_latest(0))
        .expect("empty tag prune should validate archive");

    assert!(report.is_empty());
    assert_eq!(report.retained_slot_ids, vec!["autosave", "manual"]);
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
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

    let report = archive
        .prune_slots(RuntimeSessionArchiveRetentionPolicy::keep_latest(1))
        .expect("global prune should validate archive");

    assert_eq!(report, preview);
    assert_eq!(archive.slot_ids().collect::<Vec<_>>(), vec!["slot-new"]);
}
