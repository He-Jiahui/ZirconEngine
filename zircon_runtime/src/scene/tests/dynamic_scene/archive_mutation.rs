use super::*;

#[test]
fn runtime_session_archive_renames_slots_and_updates_metadata() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world("slot-b", &source).expect("slot-b should capture"),
        RuntimeSessionSlot::from_world("slot-a", &source).expect("slot-a should capture"),
    ])
    .expect("archive should capture test slots");

    archive
        .rename_slot("slot-b", "slot-c")
        .expect("slot-b should rename");
    archive
        .update_slot_metadata(
            "slot-c",
            RuntimeSessionMetadata {
                display_name: Some("Renamed Slot".to_string()),
                tags: vec![
                    "nightly".to_string(),
                    " nightly ".to_string(),
                    "release".to_string(),
                    "".to_string(),
                ],
                ..Default::default()
            },
        )
        .expect("slot metadata should update");

    assert!(archive.slot("slot-b").is_none());
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["slot-a", "slot-c"]
    );
    let renamed = archive.slot("slot-c").expect("renamed slot should exist");
    assert_eq!(
        renamed.metadata.display_name.as_deref(),
        Some("Renamed Slot")
    );
    let expected_tags = vec!["nightly".to_string(), "release".to_string()];
    assert_eq!(renamed.metadata.tags.as_slice(), expected_tags.as_slice());

    let duplicate = archive
        .rename_slot("slot-c", "slot-a")
        .expect_err("renaming to an existing slot should fail");
    assert!(matches!(
        duplicate,
        RuntimeSessionArchiveError::DuplicateSlotId { slot_id } if slot_id == "slot-a"
    ));
}

#[test]
fn runtime_session_archive_copies_slots_with_metadata_override() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(saved_entity, "Saved Mesh")
        .expect("saved entity should be named");
    let mut archive = RuntimeSessionArchive::from_world_with_metadata(
        "autosave",
        &source,
        RuntimeSessionMetadata::default()
            .with_display_name("Autosave")
            .with_tag("nightly"),
    )
    .expect("archive should capture source slot");

    archive
        .copy_slot("autosave", " quicksave ")
        .expect("slot copy should normalize the new id");
    archive
        .copy_slot_with_metadata(
            "autosave",
            "manual",
            RuntimeSessionMetadata {
                display_name: Some("Manual Save".to_string()),
                tags: vec![
                    " beta ".to_string(),
                    "alpha".to_string(),
                    "beta".to_string(),
                ],
                ..Default::default()
            },
        )
        .expect("slot copy should accept metadata override");

    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["autosave", "manual", "quicksave"]
    );
    let quicksave = archive
        .slot("quicksave")
        .expect("copied slot should be addressable by normalized id");
    assert_eq!(quicksave.metadata.display_name.as_deref(), Some("Autosave"));
    assert_eq!(quicksave.metadata.tags, vec!["nightly"]);
    assert_eq!(quicksave.summary().entity_count, 1);

    let manual = archive
        .slot("manual")
        .expect("metadata override slot should exist");
    assert_eq!(manual.metadata.display_name.as_deref(), Some("Manual Save"));
    let expected_tags = vec!["alpha".to_string(), "beta".to_string()];
    assert_eq!(manual.metadata.tags.as_slice(), expected_tags.as_slice());
    assert_eq!(manual.summary().entity_count, 1);

    let duplicate = archive
        .copy_slot("autosave", "manual")
        .expect_err("copying to an existing slot should fail");
    assert!(matches!(
        duplicate,
        RuntimeSessionArchiveError::DuplicateSlotId { slot_id } if slot_id == "manual"
    ));

    let missing = archive
        .copy_slot("missing", "backup")
        .expect_err("copying a missing source slot should fail");
    assert!(matches!(
        missing,
        RuntimeSessionArchiveError::MissingSlot { slot_id } if slot_id == "missing"
    ));
}

#[test]
fn runtime_session_archive_merges_archives_with_explicit_conflict_policy() {
    let mut current_world = World::empty();
    current_world.spawn_node(NodeKind::Mesh);
    let mut incoming_conflict_world = World::empty();
    incoming_conflict_world.spawn_node(NodeKind::Camera);
    incoming_conflict_world.spawn_node(NodeKind::PointLight);
    let mut incoming_new_world = World::empty();
    incoming_new_world.spawn_node(NodeKind::Camera);

    let current_slot = RuntimeSessionSlot::from_world_with_metadata(
        "slot-a",
        &current_world,
        RuntimeSessionMetadata::default().with_display_name("Current"),
    )
    .expect("current slot should capture");
    let incoming_conflict = RuntimeSessionSlot::from_world_with_metadata(
        "slot-a",
        &incoming_conflict_world,
        RuntimeSessionMetadata::default().with_display_name("Incoming"),
    )
    .expect("incoming conflicting slot should capture");
    let incoming_new = RuntimeSessionSlot::from_world_with_metadata(
        "slot-b",
        &incoming_new_world,
        RuntimeSessionMetadata::default().with_display_name("New"),
    )
    .expect("incoming new slot should capture");
    let incoming =
        RuntimeSessionArchive::from_slots(vec![incoming_conflict.clone(), incoming_new.clone()])
            .expect("incoming archive should validate");

    let mut reject_archive = RuntimeSessionArchive::from_slots(vec![current_slot.clone()])
        .expect("reject archive should validate");
    let reject_error = reject_archive
        .merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::RejectConflicts)
        .expect_err("default merge policy should reject duplicate slot ids");
    assert!(matches!(
        reject_error,
        RuntimeSessionArchiveError::DuplicateSlotId { slot_id } if slot_id == "slot-a"
    ));
    assert_eq!(
        reject_archive.slot_ids().collect::<Vec<_>>(),
        vec!["slot-a"]
    );

    let mut keep_archive = RuntimeSessionArchive::from_slots(vec![current_slot.clone()])
        .expect("keep archive should validate");
    let keep_report = keep_archive
        .merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::KeepExisting)
        .expect("keep-existing merge should succeed");
    assert_eq!(keep_report.inserted_slot_ids, vec!["slot-b"]);
    assert!(keep_report.replaced_slot_ids.is_empty());
    assert_eq!(keep_report.skipped_slot_ids, vec!["slot-a"]);
    assert_eq!(
        keep_archive
            .slot("slot-a")
            .expect("kept slot should still exist")
            .summary()
            .entity_count,
        1
    );

    let mut replace_archive = RuntimeSessionArchive::from_slots(vec![current_slot])
        .expect("replace archive should validate");
    let replace_report = replace_archive
        .merge_archive(&incoming, RuntimeSessionArchiveMergePolicy::ReplaceExisting)
        .expect("replace-existing merge should succeed");
    assert_eq!(replace_report.inserted_slot_ids, vec!["slot-b"]);
    assert_eq!(replace_report.replaced_slot_ids, vec!["slot-a"]);
    assert!(replace_report.skipped_slot_ids.is_empty());
    assert_eq!(
        replace_archive
            .slot("slot-a")
            .expect("replaced slot should exist")
            .summary()
            .entity_count,
        2
    );
    assert_eq!(
        replace_archive.slot_ids().collect::<Vec<_>>(),
        vec!["slot-a", "slot-b"]
    );
}

#[test]
fn runtime_session_archive_prunes_old_slots_with_retention_policy() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "older",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Older")
                .with_updated_at_unix_millis(10),
        )
        .expect("older slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "newer",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Newer")
                .with_updated_at_unix_millis(30),
        )
        .expect("newer slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "protected",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Protected")
                .with_updated_at_unix_millis(1),
        )
        .expect("protected slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "untimed",
            &source,
            RuntimeSessionMetadata::default().with_display_name("Untimed"),
        )
        .expect("untimed slot should capture"),
    ])
    .expect("archive should validate");

    let report = archive
        .prune_slots(
            RuntimeSessionArchiveRetentionPolicy::keep_latest(2).with_protected_slot(" protected "),
        )
        .expect("retention prune should succeed");

    assert_eq!(report.retained_slot_ids, vec!["newer", "protected"]);
    assert_eq!(report.removed_slot_ids, vec!["older", "untimed"]);
    assert_eq!(report.removed_count(), 2);
    assert!(!report.is_empty());
    assert_eq!(
        archive.slot_ids().collect::<Vec<_>>(),
        vec!["newer", "protected"]
    );

    let no_limit = archive
        .prune_slots(RuntimeSessionArchiveRetentionPolicy::default())
        .expect("default retention policy should not prune");
    assert!(no_limit.is_empty());
    assert_eq!(
        no_limit.retained_slot_ids,
        vec!["newer".to_string(), "protected".to_string()]
    );
}

#[test]
fn runtime_session_archive_touches_slot_update_time_without_replacing_metadata() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Manual Save")
                .with_tag("favorite")
                .with_updated_at_unix_millis(10),
        )
        .expect("manual slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "autosave",
            &source,
            RuntimeSessionMetadata::default()
                .with_display_name("Autosave")
                .with_updated_at_unix_millis(20),
        )
        .expect("autosave slot should capture"),
    ])
    .expect("archive should validate");

    archive
        .touch_slot("manual", 30)
        .expect("manual slot timestamp should update");

    let manual = archive
        .slot("manual")
        .expect("manual slot should remain addressable");
    assert_eq!(manual.metadata.updated_at_unix_millis, Some(30));
    assert_eq!(manual.metadata.display_name.as_deref(), Some("Manual Save"));
    assert_eq!(manual.metadata.tags, vec!["favorite"]);
    assert_eq!(
        archive
            .latest_updated_slot_id()
            .expect("latest slot lookup should validate"),
        Some("manual".to_string())
    );

    let missing = archive
        .touch_slot("missing", 40)
        .expect_err("touching a missing slot should fail");
    assert!(matches!(
        missing,
        RuntimeSessionArchiveError::MissingSlot { slot_id } if slot_id == "missing"
    ));
}

#[test]
fn runtime_session_archive_diffs_slots_against_worlds() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(saved_entity, "Saved Mesh")
        .expect("saved entity should be named");
    let archive = RuntimeSessionArchive::from_world("autosave", &source)
        .expect("archive should capture source world");

    let same = archive
        .diff_slot_with_world("autosave", &source)
        .expect("same world diff should succeed");
    assert!(same.matches);
    assert_eq!(same.slot_id, "autosave");
    assert_eq!(same.slot_entity_count, 1);
    assert_eq!(same.target_entity_count, 1);

    let mut changed = source.clone();
    let extra = changed.spawn_node(NodeKind::Camera);
    changed
        .rename_node(extra, "Extra Camera")
        .expect("extra entity should be named");

    let diff = archive
        .diff_slot_with_world("autosave", &changed)
        .expect("changed world diff should succeed");
    assert!(!diff.matches);
    assert_eq!(diff.slot_entity_count, 1);
    assert_eq!(diff.target_entity_count, 2);

    let missing = archive
        .diff_slot_with_world("missing", &source)
        .expect_err("missing diff slot should fail");
    assert!(matches!(
        missing,
        RuntimeSessionArchiveError::MissingSlot { slot_id } if slot_id == "missing"
    ));
}

#[test]
fn runtime_session_archive_keeps_slot_mutation_surface_guarded() {
    let source = concat!(
        include_str!("../dynamic_scene/session/facade/mutation/rename/commit.rs"),
        include_str!("../dynamic_scene/session/facade/mutation/metadata/commit.rs"),
        include_str!("../dynamic_scene/session/facade/validation/invariants.rs"),
    );

    assert!(source.contains("pub fn rename_slot("));
    assert!(source.contains("pub fn update_slot_metadata("));
    assert!(
        !source.contains("pub fn slot_mut("),
        "direct mutable slot access would bypass archive sorting and metadata normalization"
    );
    assert!(source.contains("fn slot_mut("));
}
