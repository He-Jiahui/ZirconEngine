use super::*;

#[test]
fn runtime_session_archive_roundtrips_slots_and_restores_world() {
    let mut source = World::empty();
    let entity = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(entity, "Saved Mesh")
        .expect("source entity should be named");

    let archive = RuntimeSessionArchive::from_world_with_metadata(
        "autosave",
        &source,
        RuntimeSessionMetadata::default()
            .with_display_name("Autosave")
            .with_updated_at_unix_millis(42)
            .with_tag("runtime"),
    )
    .expect("runtime session archive should capture");
    let encoded = archive
        .to_versioned_json_pretty()
        .expect("runtime session archive should serialize");

    assert!(encoded.contains("\"format_version\": 1"));
    assert!(encoded.contains("\"slot_id\": \"autosave\""));
    assert_text_excludes_authoring_tokens(
        "runtime session archive JSON",
        &encoded,
        SERIALIZED_AUTHORING_TOKENS,
    );

    let decoded = RuntimeSessionArchive::from_versioned_json(&encoded)
        .expect("runtime session archive should reload");
    assert_eq!(
        decoded
            .slot("autosave")
            .expect("autosave slot should exist")
            .metadata
            .display_name
            .as_deref(),
        Some("Autosave")
    );
    let restored = decoded
        .restore_slot_to_empty_world("autosave")
        .expect("slot should restore into an empty world");

    assert_eq!(
        restored
            .find_node(entity)
            .expect("restored entity should keep the free source id")
            .name,
        "Saved Mesh"
    );
}

#[test]
fn runtime_session_archive_rejects_duplicate_slots() {
    let source = World::empty();
    let first = RuntimeSessionSlot::from_world("slot", &source).expect("first slot should capture");
    let second =
        RuntimeSessionSlot::from_world("slot", &source).expect("second slot should capture");

    let error = RuntimeSessionArchive::from_slots(vec![first, second])
        .expect_err("duplicate slot ids should be rejected");

    assert!(matches!(
        error,
        RuntimeSessionArchiveError::DuplicateSlotId { slot_id } if slot_id == "slot"
    ));
}

#[test]
fn runtime_session_archive_rejects_unsupported_embedded_scene_versions() {
    let source = World::empty();
    let mut slot = RuntimeSessionSlot::from_world("slot", &source)
        .expect("runtime session slot should capture");
    slot.scene.format_version = 999;

    let error = RuntimeSessionArchive::from_slots(vec![slot.clone()])
        .expect_err("unsupported embedded scene version should be rejected");
    assert!(matches!(
        error,
        RuntimeSessionArchiveError::DynamicScene(
            crate::scene::DynamicSceneError::UnsupportedFormatVersion {
                expected: 1,
                actual: 999,
            }
        )
    ));

    let json = serde_json::to_string(&RuntimeSessionArchive {
        format_version: 1,
        slots: vec![slot],
    })
    .expect("bad archive fixture should serialize");
    let error = RuntimeSessionArchive::from_versioned_json(&json)
        .expect_err("unsupported embedded scene version should fail during load");
    assert!(matches!(
        error,
        RuntimeSessionArchiveError::DynamicScene(
            crate::scene::DynamicSceneError::UnsupportedFormatVersion {
                expected: 1,
                actual: 999,
            }
        )
    ));
}

#[test]
fn runtime_session_archive_rejects_unsupported_slots_on_push_and_upsert() {
    let source = World::empty();
    let mut push_archive = RuntimeSessionArchive::empty();
    let mut push_slot =
        RuntimeSessionSlot::from_world("slot", &source).expect("push slot should capture");
    push_slot.scene.format_version = 999;

    let push_error = push_archive
        .push_slot(push_slot)
        .expect_err("push_slot should validate embedded dynamic scene");
    assert!(matches!(
        push_error,
        RuntimeSessionArchiveError::DynamicScene(
            crate::scene::DynamicSceneError::UnsupportedFormatVersion {
                expected: 1,
                actual: 999
            }
        )
    ));

    let mut upsert_archive = RuntimeSessionArchive::empty();
    let mut upsert_slot =
        RuntimeSessionSlot::from_world("slot", &source).expect("upsert slot should capture");
    upsert_slot.scene.format_version = 999;

    let upsert_error = upsert_archive
        .upsert_slot(upsert_slot)
        .expect_err("upsert_slot should validate embedded dynamic scene");
    assert!(matches!(
        upsert_error,
        RuntimeSessionArchiveError::DynamicScene(
            crate::scene::DynamicSceneError::UnsupportedFormatVersion {
                expected: 1,
                actual: 999
            }
        )
    ));
}

#[test]
fn runtime_session_archive_rejects_non_canonical_slot_ids() {
    let source = World::empty();
    let mut slot = RuntimeSessionSlot::from_world("slot", &source)
        .expect("runtime session slot should capture");
    slot.slot_id = " slot ".to_string();

    let error = RuntimeSessionArchive::from_slots(vec![slot.clone()])
        .expect_err("non-canonical slot ids should be rejected");
    assert!(matches!(
        error,
        RuntimeSessionArchiveError::NonCanonicalSlotId { slot_id, canonical }
            if slot_id == " slot " && canonical == "slot"
    ));

    let json = serde_json::to_string(&RuntimeSessionArchive {
        format_version: 1,
        slots: vec![slot],
    })
    .expect("bad archive fixture should serialize");
    let error = RuntimeSessionArchive::from_versioned_json(&json)
        .expect_err("non-canonical slot ids should fail during load");
    assert!(matches!(
        error,
        RuntimeSessionArchiveError::NonCanonicalSlotId { slot_id, canonical }
            if slot_id == " slot " && canonical == "slot"
    ));
}

#[test]
fn runtime_session_archive_serializes_manual_slots_in_canonical_order() {
    let source = World::empty();
    let archive = RuntimeSessionArchive {
        format_version: 1,
        slots: vec![
            RuntimeSessionSlot::from_world("slot-b", &source).expect("slot-b should capture"),
            RuntimeSessionSlot::from_world("slot-a", &source).expect("slot-a should capture"),
        ],
    };

    let encoded = archive
        .to_versioned_json_pretty()
        .expect("manual archive should serialize");
    let slot_a = encoded
        .find("\"slot_id\": \"slot-a\"")
        .expect("slot-a should be serialized");
    let slot_b = encoded
        .find("\"slot_id\": \"slot-b\"")
        .expect("slot-b should be serialized");

    assert!(
        slot_a < slot_b,
        "manual archive serialization should sort slots by slot id"
    );
}

#[test]
fn runtime_session_archive_normalizes_metadata_tags_for_manifest_and_json() {
    let source = World::empty();
    let archive = RuntimeSessionArchive {
        format_version: 1,
        slots: vec![RuntimeSessionSlot {
            slot_id: "slot".to_string(),
            metadata: RuntimeSessionMetadata {
                tags: vec![
                    " beta ".to_string(),
                    "alpha".to_string(),
                    "".to_string(),
                    "beta".to_string(),
                    "  ".to_string(),
                ],
                ..Default::default()
            },
            scene: DynamicScene::from_world(&source).expect("source world should export"),
        }],
    };

    let manifest = archive
        .manifest()
        .expect("manifest should normalize metadata tags");
    let expected_tags = vec!["alpha".to_string(), "beta".to_string()];
    assert_eq!(
        manifest
            .slot("slot")
            .expect("slot summary should exist")
            .metadata
            .tags
            .as_slice(),
        expected_tags.as_slice()
    );

    let encoded = archive
        .to_versioned_json_pretty()
        .expect("manual archive should serialize with normalized metadata");
    let decoded = RuntimeSessionArchive::from_versioned_json(&encoded)
        .expect("encoded archive should reload");
    assert_eq!(
        decoded
            .slot("slot")
            .expect("slot should reload")
            .metadata
            .tags
            .as_slice(),
        expected_tags.as_slice()
    );
}
