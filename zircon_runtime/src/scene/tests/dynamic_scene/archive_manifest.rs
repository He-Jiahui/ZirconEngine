use super::*;

#[test]
fn runtime_session_archive_statistics_summarizes_slots_without_restoring_worlds() {
    let empty = World::empty();
    let mut populated = World::empty();
    populated.spawn_node(NodeKind::Mesh);
    populated.spawn_node(NodeKind::Camera);
    let mut single = World::empty();
    single.spawn_node(NodeKind::PointLight);

    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "empty",
            &empty,
            RuntimeSessionMetadata::default().with_updated_at_unix_millis(20),
        )
        .expect("empty slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "populated",
            &populated,
            RuntimeSessionMetadata::default().with_updated_at_unix_millis(40),
        )
        .expect("populated slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "untimed",
            &single,
            RuntimeSessionMetadata::default().with_display_name("Untimed"),
        )
        .expect("untimed slot should capture"),
    ])
    .expect("archive should validate");

    let statistics = archive
        .statistics()
        .expect("archive statistics should be available");

    assert!(!statistics.is_empty());
    assert_eq!(statistics.format_version, 1);
    assert_eq!(statistics.slot_count, 3);
    assert_eq!(statistics.total_entity_count, 3);
    assert_eq!(statistics.total_resource_count, 0);
    assert_eq!(statistics.max_slot_entity_count, 2);
    assert_eq!(statistics.max_slot_resource_count, 0);
    assert_eq!(statistics.earliest_updated_at_unix_millis, Some(20));
    assert_eq!(statistics.latest_updated_at_unix_millis, Some(40));
    assert_eq!(statistics.untimed_slot_count, 1);
    assert!(statistics.has_untimed_slots());

    let empty_statistics = RuntimeSessionArchive::empty()
        .statistics()
        .expect("empty archive statistics should be available");
    assert!(empty_statistics.is_empty());
    assert_eq!(empty_statistics.slot_count, 0);
    assert_eq!(empty_statistics.untimed_slot_count, 0);
    assert_eq!(empty_statistics.latest_updated_at_unix_millis, None);
}

#[test]
fn runtime_session_archive_selects_latest_and_oldest_updated_slots_without_restoring_worlds() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "untimed",
            &source,
            RuntimeSessionMetadata::default().with_display_name("Untimed"),
        )
        .expect("untimed slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "early",
            &source,
            RuntimeSessionMetadata::default().with_updated_at_unix_millis(10),
        )
        .expect("early slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "late-b",
            &source,
            RuntimeSessionMetadata::default().with_updated_at_unix_millis(40),
        )
        .expect("late-b slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "late-c",
            &source,
            RuntimeSessionMetadata::default().with_updated_at_unix_millis(40),
        )
        .expect("late-c slot should capture"),
    ])
    .expect("archive should validate");

    let manifest = archive.manifest().expect("manifest should be available");
    assert_eq!(
        manifest
            .latest_updated_slot()
            .expect("latest slot should exist")
            .slot_id,
        "late-c"
    );
    assert_eq!(
        manifest
            .oldest_updated_slot()
            .expect("oldest slot should exist")
            .slot_id,
        "untimed"
    );
    assert_eq!(
        archive
            .latest_updated_slot_id()
            .expect("latest slot id should be available")
            .as_deref(),
        Some("late-c")
    );
    assert_eq!(
        archive
            .oldest_updated_slot_id()
            .expect("oldest slot id should be available")
            .as_deref(),
        Some("untimed")
    );

    let empty = RuntimeSessionArchive::empty();
    let empty_manifest = empty
        .manifest()
        .expect("empty manifest should be available");
    assert!(empty_manifest.latest_updated_slot().is_none());
    assert!(empty_manifest.oldest_updated_slot().is_none());
    assert_eq!(
        empty
            .latest_updated_slot_id()
            .expect("empty latest lookup should succeed"),
        None
    );
    assert_eq!(
        empty
            .oldest_updated_slot_id()
            .expect("empty oldest lookup should succeed"),
        None
    );
}

#[test]
fn runtime_session_archive_manifest_summarizes_sorted_slots() {
    let empty = World::empty();
    let mut populated = World::empty();
    populated.spawn_node(NodeKind::Mesh);
    populated.spawn_node(NodeKind::Camera);

    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "slot-b",
            &populated,
            RuntimeSessionMetadata::default().with_display_name("Populated"),
        )
        .expect("populated slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "slot-a",
            &empty,
            RuntimeSessionMetadata::default().with_display_name("Empty"),
        )
        .expect("empty slot should capture"),
    ])
    .expect("archive should accept unique slots");

    let manifest = archive
        .manifest()
        .expect("archive manifest should be available");

    assert_eq!(manifest.slot_count(), 2);
    assert_eq!(
        manifest.slot_ids().collect::<Vec<_>>(),
        vec!["slot-a", "slot-b"]
    );
    assert_eq!(
        manifest
            .slot("slot-a")
            .expect("slot-a should have a summary")
            .entity_count,
        0
    );
    let populated_summary = manifest
        .slot("slot-b")
        .expect("slot-b should have a summary");
    assert_eq!(populated_summary.entity_count, 2);
    assert_eq!(populated_summary.resource_count, 0);
    assert_eq!(
        populated_summary.metadata.display_name.as_deref(),
        Some("Populated")
    );
}

#[test]
fn runtime_session_archive_manifest_filters_slots_without_restoring_worlds() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "slot-c",
            &source,
            RuntimeSessionMetadata {
                display_name: Some("Nightly Castle".to_string()),
                tags: vec!["nightly".to_string(), "castle".to_string()],
                ..Default::default()
            },
        )
        .expect("slot-c should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "slot-a",
            &source,
            RuntimeSessionMetadata {
                display_name: Some("Manual Castle".to_string()),
                tags: vec!["manual".to_string(), "castle".to_string()],
                ..Default::default()
            },
        )
        .expect("slot-a should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "slot-b",
            &source,
            RuntimeSessionMetadata {
                display_name: Some("Manual Arena".to_string()),
                tags: vec!["manual".to_string(), "arena".to_string()],
                ..Default::default()
            },
        )
        .expect("slot-b should capture"),
    ])
    .expect("archive should accept searchable slots");

    let manifest = archive
        .manifest()
        .expect("archive manifest should be available");

    assert_eq!(
        manifest
            .slots_with_tag(" manual ")
            .map(|slot| slot.slot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["slot-a", "slot-b"]
    );
    assert_eq!(
        manifest
            .slots_matching_display_name("Castle")
            .map(|slot| slot.slot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["slot-a", "slot-c"]
    );
    assert_eq!(manifest.slots_with_tag("missing").count(), 0);
    assert_eq!(manifest.slots_matching_display_name("  ").count(), 0);
}

#[test]
fn runtime_session_archive_selects_latest_and_oldest_updated_slots_by_tag() {
    let source = World::empty();
    let archive = RuntimeSessionArchive::from_slots(vec![
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-a",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag("manual")
                .with_updated_at_unix_millis(10),
        )
        .expect("manual-a slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-b",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag("manual")
                .with_updated_at_unix_millis(40),
        )
        .expect("manual-b slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-c",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag("manual")
                .with_updated_at_unix_millis(40),
        )
        .expect("manual-c slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "autosave",
            &source,
            RuntimeSessionMetadata::default()
                .with_tag("autosave")
                .with_updated_at_unix_millis(100),
        )
        .expect("autosave slot should capture"),
        RuntimeSessionSlot::from_world_with_metadata(
            "manual-untimed",
            &source,
            RuntimeSessionMetadata::default().with_tag(" manual "),
        )
        .expect("manual-untimed slot should capture"),
    ])
    .expect("archive should accept tagged slots");

    let manifest = archive
        .manifest()
        .expect("archive manifest should be available");
    assert_eq!(
        manifest
            .latest_updated_slot_with_tag(" manual ")
            .expect("latest manual slot should exist")
            .slot_id,
        "manual-c"
    );
    assert_eq!(
        manifest
            .oldest_updated_slot_with_tag("manual")
            .expect("oldest manual slot should exist")
            .slot_id,
        "manual-untimed"
    );
    assert!(manifest.latest_updated_slot_with_tag("  ").is_none());
    assert!(manifest.oldest_updated_slot_with_tag("missing").is_none());
    assert_eq!(
        archive
            .latest_updated_slot_id_with_tag("manual")
            .expect("latest tagged slot lookup should validate"),
        Some("manual-c".to_string())
    );
    assert_eq!(
        archive
            .oldest_updated_slot_id_with_tag("manual")
            .expect("oldest tagged slot lookup should validate"),
        Some("manual-untimed".to_string())
    );
}

#[test]
fn runtime_session_archive_upsert_replaces_slot_summary() {
    let empty = World::empty();
    let mut populated = World::empty();
    populated.spawn_node(NodeKind::Mesh);

    let mut archive = RuntimeSessionArchive::from_world("autosave", &empty)
        .expect("archive should capture initial autosave");
    assert!(archive.contains_slot("autosave"));
    assert_eq!(archive.slot_count(), 1);

    archive
        .capture_world_slot(
            "autosave",
            &populated,
            RuntimeSessionMetadata::default().with_display_name("Updated"),
        )
        .expect("autosave should be replaced");
    let manifest = archive
        .manifest()
        .expect("archive manifest should reflect replacement");
    let autosave = manifest
        .slot("autosave")
        .expect("autosave summary should exist");

    assert_eq!(manifest.slot_count(), 1);
    assert_eq!(autosave.entity_count, 1);
    assert_eq!(autosave.metadata.display_name.as_deref(), Some("Updated"));
}
