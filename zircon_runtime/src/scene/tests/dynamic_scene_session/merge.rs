use std::fs;
use std::io::ErrorKind;

use crate::scene::{
    NodeKind, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionMetadata, RuntimeSessionSlot, World,
};

use super::{temporary_archive_leftovers, unique_temp_root};

#[test]
fn runtime_session_archive_merge_preview_and_keep_existing_commit_are_side_effect_free() {
    let target_manual =
        slot_with_nodes("manual", "Target Manual", "target", 10, &[NodeKind::Camera]);
    let target_autosave = slot_with_nodes(
        "autosave",
        "Target Autosave",
        "target",
        20,
        &[NodeKind::Mesh],
    );
    let incoming_manual = slot_with_nodes(
        "manual",
        "Incoming Manual",
        "incoming",
        80,
        &[NodeKind::Mesh, NodeKind::PointLight],
    );
    let incoming_bonus =
        slot_with_nodes("bonus", "Incoming Bonus", "incoming", 30, &[NodeKind::Cube]);
    let target =
        RuntimeSessionArchive::from_slots(vec![target_manual.clone(), target_autosave.clone()])
            .expect("target archive should validate");
    let incoming =
        RuntimeSessionArchive::from_slots(vec![incoming_manual.clone(), incoming_bonus.clone()])
            .expect("incoming archive should validate");

    let preview = target
        .preview_merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)
        .expect("keep-existing merge preview should validate");
    assert_eq!(preview.inserted_slot_ids, vec!["bonus"]);
    assert_eq!(preview.skipped_slot_ids, vec!["manual"]);
    assert!(preview.replaced_slot_ids.is_empty());
    assert_eq!(
        target.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual"]
    );
    assert_eq!(
        incoming.slot_ids().collect::<Vec<_>>(),
        vec!["bonus", "manual"]
    );
    assert_eq!(
        target
            .slot("manual")
            .expect("target manual slot should remain after preview")
            .metadata
            .display_name
            .as_deref(),
        Some("Target Manual")
    );

    let mut committed = target;
    let report = committed
        .merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)
        .expect("keep-existing merge should insert only non-conflicting slots");
    assert_eq!(report, preview);
    assert_eq!(
        committed.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "bonus", "manual"]
    );
    let kept = committed
        .manifest()
        .expect("committed archive should project manifest")
        .slot("manual")
        .expect("kept manual summary should exist")
        .clone();
    assert_eq!(kept.entity_count, 1);
    assert_eq!(kept.metadata.display_name.as_deref(), Some("Target Manual"));
    assert_eq!(kept.metadata.tags, vec!["target"]);
    assert_eq!(
        incoming.slot_ids().collect::<Vec<_>>(),
        vec!["bonus", "manual"]
    );
}

#[test]
fn runtime_session_archive_path_merge_preview_commit_and_same_path_guard_are_atomic() {
    let target = RuntimeSessionArchive::from_slots(vec![
        slot_with_nodes("manual", "Target Manual", "target", 10, &[NodeKind::Camera]),
        slot_with_nodes(
            "autosave",
            "Target Autosave",
            "target",
            20,
            &[NodeKind::Mesh],
        ),
    ])
    .expect("target archive should validate");
    let incoming = RuntimeSessionArchive::from_slots(vec![
        slot_with_nodes(
            "manual",
            "Incoming Manual",
            "incoming",
            80,
            &[NodeKind::Mesh, NodeKind::PointLight],
        ),
        slot_with_nodes("bonus", "Incoming Bonus", "incoming", 30, &[NodeKind::Cube]),
    ])
    .expect("incoming archive should validate");
    let root = unique_temp_root("runtime_session_merge_behavior");
    let target_path = root.join("target").join("archive.zrsession.json");
    let source_path = root.join("source").join("archive.zrsession.json");
    target
        .save_to_path_atomically(&target_path)
        .expect("target archive should save before merge");
    incoming
        .save_to_path_atomically(&source_path)
        .expect("source archive should save before merge");
    let target_payload =
        fs::read_to_string(&target_path).expect("target archive payload should be readable");

    let preview = RuntimeSessionArchive::preview_merge_archive_at_path(
        &target_path,
        &incoming,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect("loaded incoming merge preview should validate");
    assert_eq!(preview.inserted_slot_ids, vec!["bonus"]);
    assert_eq!(preview.replaced_slot_ids, vec!["manual"]);
    assert!(preview.skipped_slot_ids.is_empty());
    assert_eq!(
        fs::read_to_string(&target_path).expect("target payload should remain after preview"),
        target_payload
    );

    let same_path_preview = RuntimeSessionArchive::preview_merge_archive_from_path_at_path(
        &target_path,
        &target_path,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect_err("source-path preview should reject same archive path before loading");
    assert_same_path_merge_rejected(same_path_preview);
    assert_eq!(
        fs::read_to_string(&target_path).expect("target payload should remain after rejection"),
        target_payload
    );

    let report = RuntimeSessionArchive::merge_archive_from_path_at_path_atomically(
        &target_path,
        &source_path,
        RuntimeSessionArchiveMergePolicy::ReplaceExisting,
    )
    .expect("source archive path should merge into target atomically");
    assert_eq!(report, preview);

    let loaded = RuntimeSessionArchive::load_from_path(&target_path)
        .expect("merged archive should reload after atomic commit");
    assert_eq!(
        loaded.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "bonus", "manual"]
    );
    let replaced = loaded
        .manifest()
        .expect("merged archive should project manifest")
        .slot("manual")
        .expect("replaced manual summary should exist")
        .clone();
    assert_eq!(replaced.entity_count, 2);
    assert_eq!(
        replaced.metadata.display_name.as_deref(),
        Some("Incoming Manual")
    );
    assert_eq!(replaced.metadata.tags, vec!["incoming"]);
    assert!(temporary_archive_leftovers(
        target_path
            .parent()
            .expect("target path should have parent")
    )
    .is_empty());
    assert!(temporary_archive_leftovers(
        source_path
            .parent()
            .expect("source path should have parent")
    )
    .is_empty());

    let committed_payload =
        fs::read_to_string(&target_path).expect("committed target payload should be readable");
    let same_path_commit = RuntimeSessionArchive::merge_archive_from_path_at_path_atomically(
        &target_path,
        &target_path,
        RuntimeSessionArchiveMergePolicy::KeepExisting,
    )
    .expect_err("source-path commit should reject same archive path before mutation");
    assert_same_path_merge_rejected(same_path_commit);
    assert_eq!(
        fs::read_to_string(&target_path)
            .expect("target payload should remain after same-path commit rejection"),
        committed_payload
    );

    let _ = fs::remove_dir_all(root);
}

fn slot_with_nodes(
    slot_id: &str,
    display_name: &str,
    tag: &str,
    updated_at_unix_millis: u64,
    node_kinds: &[NodeKind],
) -> RuntimeSessionSlot {
    let mut world = World::empty();
    for node_kind in node_kinds {
        world.spawn_node(*node_kind);
    }
    RuntimeSessionSlot::from_world_with_metadata(
        slot_id,
        &world,
        RuntimeSessionMetadata::default()
            .with_display_name(display_name)
            .with_tag(tag)
            .with_updated_at_unix_millis(updated_at_unix_millis),
    )
    .expect("merge slot should capture")
}

fn assert_same_path_merge_rejected(error: RuntimeSessionArchiveError) {
    match error {
        RuntimeSessionArchiveError::Io(error) => {
            assert_eq!(error.kind(), ErrorKind::InvalidInput);
            assert!(
                error
                    .to_string()
                    .contains("target archive path must differ from source archive path"),
                "unexpected same-path merge error: {error}"
            );
        }
        other => panic!("expected same-path merge to return I/O invalid input, got {other:?}"),
    }
}
