use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue, ReflectObjectAddress,
    ReflectReadRequest, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectedValue,
};

use crate::core::framework::physics::PhysicsWorldStepPlan;
use crate::plugin::ComponentTypeDescriptor;
use crate::scene::ecs::Resource;
use crate::scene::{
    DefaultLevelManager, DynamicScene, DynamicSceneError, NodeKind, ReflectResource,
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionMetadata, RuntimeSessionSlot, SceneError,
    ScenePatch, World,
};

use super::authoring_boundary::{
    assert_text_excludes_authoring_tokens, SERIALIZED_AUTHORING_TOKENS,
};

const CLOUD_LAYER_TYPE_PATH: &str = "weather.Component.CloudLayer";
const FRAME_COUNTER_TYPE_PATH: &str = "zircon_runtime::scene::tests::dynamic_scene::FrameCounter";

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter {
    value: u32,
}

impl Resource for FrameCounter {}

#[test]
fn dynamic_scene_roundtrips_reflected_components_with_entity_remap() {
    let mut source = World::empty();
    source
        .register_component_type(cloud_layer_descriptor())
        .expect("dynamic descriptor should register");
    let parent = source.spawn_node(NodeKind::Mesh);
    let child = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(parent, "Weather Root")
        .expect("parent should be named");
    source
        .rename_node(child, "Cloud")
        .expect("child should be named");
    source
        .set_parent_checked(child, Some(parent))
        .expect("child should be parented");
    source
        .set_dynamic_component(
            child,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.75, "label": "storm front" }),
        )
        .expect("dynamic component should attach");

    let encoded = serde_json::to_string(
        &DynamicScene::from_world(&source).expect("source world should export"),
    )
    .expect("dynamic scene should serialize");
    assert!(encoded.contains("\"format_version\":1"));
    assert!(encoded.contains(CLOUD_LAYER_TYPE_PATH));
    assert_text_excludes_authoring_tokens(
        "dynamic scene JSON",
        &encoded,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let scene: DynamicScene =
        serde_json::from_str(&encoded).expect("dynamic scene should deserialize");

    let mut target = World::empty();
    target
        .register_component_type(cloud_layer_descriptor())
        .expect("target descriptor should register");
    let collision = target.spawn_node(NodeKind::Mesh);
    assert_eq!(collision, parent);

    let remap = scene
        .spawn_into(&mut target)
        .expect("scene should spawn into target world");
    let mapped_parent = remap
        .get(parent)
        .expect("parent should have a target mapping");
    let mapped_child = remap
        .get(child)
        .expect("child should have a target mapping");

    assert_ne!(mapped_parent, parent);
    assert_eq!(target.parent_of(mapped_child), Some(mapped_parent));
    assert_eq!(
        target
            .find_node(mapped_child)
            .expect("mapped child should exist")
            .name,
        "Cloud"
    );
    assert_eq!(
        target.dynamic_component(mapped_child, CLOUD_LAYER_TYPE_PATH),
        Some(&json!({ "coverage": 0.75, "label": "storm front" }))
    );
    assert_eq!(
        target
            .reflect_read(ReflectReadRequest::new(
                ReflectObjectAddress::component(mapped_child, CLOUD_LAYER_TYPE_PATH)
                    .expect("component address should be valid"),
                "coverage",
            ))
            .expect("spawned dynamic field should read through reflection")
            .field,
        ReflectFieldValue::new("coverage", ReflectedValue::Scalar(0.75))
    );
}

#[test]
fn scene_patch_applies_reflected_resources() {
    let mut source = World::empty();
    register_frame_counter_resource(&mut source);
    source.insert_resource(FrameCounter { value: 7 });

    let patch = ScenePatch::from_scene(
        DynamicScene::from_world(&source).expect("resource world should export"),
    );

    let mut target = World::empty();
    register_frame_counter_resource(&mut target);
    target.insert_resource(FrameCounter { value: 0 });

    let preview = patch
        .preview_apply(&target)
        .expect("resource patch preview should inspect target resources");
    assert_eq!(preview.resource_count, 1);
    assert_eq!(preview.resources.len(), 1);
    assert_eq!(preview.resources[0].type_path, FRAME_COUNTER_TYPE_PATH);
    assert!(preview.resources[0].already_present);
    assert!(!preview.resources[0].can_create_on_apply);
    assert_eq!(preview.resources[0].field_count, 1);
    assert!(preview.resources_requiring_creation().next().is_none());
    assert_eq!(
        target
            .get_resource::<FrameCounter>()
            .expect("preview should not mutate target resource")
            .value,
        0
    );

    let remap = patch
        .apply(&mut target)
        .expect("resource patch should apply");

    assert!(remap.is_empty());
    assert_eq!(
        target
            .get_resource::<FrameCounter>()
            .expect("target resource should still exist")
            .value,
        7
    );

    let mut target_with_ensure = World::empty();
    register_frame_counter_resource_with_ensure(&mut target_with_ensure);
    let preview_with_ensure = patch
        .preview_apply(&target_with_ensure)
        .expect("resource patch preview should accept ensure-backed resources");
    assert_eq!(preview_with_ensure.resource_count, 1);
    assert_eq!(preview_with_ensure.resources.len(), 1);
    assert_eq!(
        preview_with_ensure.resources[0].type_path,
        FRAME_COUNTER_TYPE_PATH
    );
    assert!(!preview_with_ensure.resources[0].already_present);
    assert!(preview_with_ensure.resources[0].can_create_on_apply);
    assert_eq!(preview_with_ensure.resources[0].field_count, 1);
    assert_eq!(
        preview_with_ensure
            .resources_requiring_creation()
            .map(|resource| resource.type_path.as_str())
            .collect::<Vec<_>>(),
        [FRAME_COUNTER_TYPE_PATH]
    );
    assert!(target_with_ensure.get_resource::<FrameCounter>().is_none());

    patch
        .apply(&mut target_with_ensure)
        .expect("ensure-backed resource patch should apply");
    assert_eq!(
        target_with_ensure
            .get_resource::<FrameCounter>()
            .expect("apply should create target resource through ensure")
            .value,
        7
    );
}

#[test]
fn scene_patch_preview_reports_remaps_without_mutating_target_world() {
    let mut source = World::empty();
    source
        .register_component_type(cloud_layer_descriptor())
        .expect("source descriptor should register");
    let parent = source.spawn_node(NodeKind::Mesh);
    let child = source.spawn_node(NodeKind::Mesh);
    source
        .set_parent_checked(child, Some(parent))
        .expect("child should be parented");
    source
        .set_dynamic_component(
            child,
            CLOUD_LAYER_TYPE_PATH,
            json!({ "coverage": 0.5, "label": "preview" }),
        )
        .expect("dynamic component should attach");
    let patch = ScenePatch::from_world(&source).expect("source world should export");

    let mut target = World::empty();
    let collision = target.spawn_node(NodeKind::Mesh);
    assert_eq!(collision, parent);
    let target_before = DynamicScene::from_world(&target).expect("target should export");

    let preview = patch
        .preview_apply(&target)
        .expect("patch preview should resolve remaps");

    assert_eq!(preview.component_type_count, 1);
    assert_eq!(preview.existing_component_type_count, 0);
    assert_eq!(preview.new_component_type_count, 1);
    assert_eq!(preview.component_types.len(), 1);
    assert_eq!(preview.component_types[0].type_id, CLOUD_LAYER_TYPE_PATH);
    assert_eq!(preview.component_types[0].plugin_id, "weather");
    assert_eq!(preview.component_types[0].display_name, "Cloud Layer");
    assert!(!preview.component_types[0].already_registered);
    assert_eq!(preview.component_instance_count, 1);
    assert_eq!(preview.entity_count, 2);
    assert_eq!(preview.resource_count, 0);
    assert!(preview.resources.is_empty());
    assert_eq!(preview.target_entity_count, 1);
    assert_eq!(preview.preserved_entity_count, 0);
    assert_eq!(preview.remapped_entity_count, 2);
    assert_eq!(preview.entity_remaps.len(), 2);
    assert_eq!(preview.entity_remaps[0].source_entity, parent);
    assert_eq!(preview.entity_remaps[0].target_entity, parent + 1);
    assert_eq!(preview.entity_remaps[1].source_entity, child);
    assert_eq!(preview.entity_remaps[1].target_entity, child + 1);
    assert!(preview.has_entity_remaps());
    assert!(preview.has_new_component_types());
    assert_eq!(
        preview
            .new_component_types()
            .map(|component_type| component_type.type_id.as_str())
            .collect::<Vec<_>>(),
        [CLOUD_LAYER_TYPE_PATH]
    );
    assert_eq!(
        DynamicScene::from_world(&target).expect("target should export after preview"),
        target_before
    );
    assert!(!target.contains_entity(child));
}

#[test]
fn dynamic_scene_world_mutation_preserves_scene_error_source() {
    let mut scene = DynamicScene::empty();
    scene.component_types.push(
        ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
            .with_property("", "Scalar", true),
    );

    let error = scene
        .spawn_into(&mut World::empty())
        .expect_err("invalid dynamic component registration should preserve scene error source");

    assert!(matches!(
        error,
        DynamicSceneError::WorldMutation(SceneError::Reflect(
            ReflectError::InvalidRegistration { type_path, reason }
        )) if type_path == CLOUD_LAYER_TYPE_PATH
            && reason == "dynamic component field name must not be empty"
    ));
}

#[test]
fn versioned_json_migrates_legacy_world_project_documents() {
    let mut legacy = World::empty();
    let entity = legacy.spawn_node(NodeKind::Mesh);
    legacy
        .rename_node(entity, "Legacy Mesh")
        .expect("legacy entity should be named");
    let legacy_json = serde_json::to_string(&json!({
        "format_version": 2,
        "world": legacy,
    }))
    .expect("legacy project document should serialize");

    let scene =
        DynamicScene::from_versioned_json(&legacy_json).expect("legacy world should migrate");
    let migrated = scene
        .entities
        .iter()
        .find(|entity| entity.source_entity == 1)
        .expect("legacy entity should be migrated");

    assert_eq!(migrated.record.name, "Legacy Mesh");
    assert_eq!(migrated.record.kind, NodeKind::Mesh);

    let encoded = scene
        .to_versioned_json_pretty()
        .expect("dynamic scene should write versioned JSON");
    assert_text_excludes_authoring_tokens(
        "versioned dynamic scene JSON",
        &encoded,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let decoded =
        DynamicScene::from_versioned_json(&encoded).expect("dynamic scene JSON should reload");
    assert_eq!(decoded, scene);
}

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

#[test]
fn runtime_session_archive_restores_slot_into_level_and_resets_runtime_state() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(saved_entity, "Restored Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_world_with_metadata(
        "level-slot",
        &source,
        RuntimeSessionMetadata::default()
            .with_display_name("Restored Level")
            .with_tag("level"),
    )
    .expect("archive should capture level slot");

    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    let stale_entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Camera);
        world
            .rename_node(entity, "Stale Camera")
            .expect("stale entity should be named");
        entity
    });
    level.record_physics_step(
        PhysicsWorldStepPlan {
            steps: 1,
            step_seconds: 0.016,
            remaining_seconds: 0.0,
            interpolation_alpha: 0.0,
        },
        Vec::new(),
        Vec::new(),
    );

    let report = archive
        .restore_slot_into_level("level-slot", &level)
        .expect("slot should restore directly into the level");

    assert_eq!(report.slot_id, "level-slot");
    assert_eq!(report.entity_count, 1);
    assert_eq!(
        level.metadata().display_name.as_deref(),
        Some("Restored Level")
    );
    assert!(level.last_physics_step_plan().is_none());
    level.with_world(|world| {
        assert!(world.find_node(stale_entity).is_none());
        assert_eq!(
            world
                .find_node(saved_entity)
                .expect("restored entity should exist")
                .name,
            "Restored Mesh"
        );
    });
}

#[test]
fn runtime_session_archive_applies_slot_to_live_level_with_entity_remap() {
    let mut source = World::empty();
    let saved_entity = source.spawn_node(NodeKind::Mesh);
    source
        .rename_node(saved_entity, "Instanced Mesh")
        .expect("source entity should be named");
    let archive = RuntimeSessionArchive::from_world("prefab-slot", &source)
        .expect("archive should capture prefab slot");

    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), Default::default());
    let existing_entity = level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Camera);
        world
            .rename_node(entity, "Live Camera")
            .expect("existing entity should be named");
        entity
    });
    assert_eq!(existing_entity, saved_entity);

    let remap = archive
        .apply_slot_to_level("prefab-slot", &level)
        .expect("slot should apply into the live level");
    let mapped_entity = remap
        .get(saved_entity)
        .expect("source entity should be remapped after id collision");

    assert_ne!(mapped_entity, existing_entity);
    level.with_world(|world| {
        assert_eq!(
            world
                .find_node(existing_entity)
                .expect("existing entity should remain")
                .name,
            "Live Camera"
        );
        assert_eq!(
            world
                .find_node(mapped_entity)
                .expect("mapped entity should be spawned")
                .name,
            "Instanced Mesh"
        );
    });
}

fn cloud_layer_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(CLOUD_LAYER_TYPE_PATH, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}

fn register_frame_counter_resource(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(frame_counter_registration(), frame_counter_adapter())
        .expect("frame counter resource registration should be accepted");
}

fn register_frame_counter_resource_with_ensure(world: &mut World) {
    world
        .type_registry_mut_for_tests()
        .register_resource(
            frame_counter_registration(),
            frame_counter_adapter_with_ensure(),
        )
        .expect("ensure-backed frame counter resource registration should be accepted");
}

fn frame_counter_registration() -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(FRAME_COUNTER_TYPE_PATH, "FrameCounter")
            .expect("frame counter type path should be valid"),
        "Frame Counter",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            "value",
            "Unsigned",
            ReflectEditorHint::Unsigned,
        )]),
        ReflectSerializationStrategy::ResourceHandle,
    )
    .as_resource()
    .with_remote_visible(true)
}

fn frame_counter_adapter() -> ReflectResource {
    ReflectResource {
        ensure: None,
        contains: frame_counter_contains,
        read_field: frame_counter_read_field,
        read_fields: frame_counter_read_fields,
        write_field: frame_counter_write_field,
    }
}

fn frame_counter_adapter_with_ensure() -> ReflectResource {
    ReflectResource {
        ensure: Some(frame_counter_ensure),
        ..frame_counter_adapter()
    }
}

fn frame_counter_ensure(world: &mut World) -> Result<bool, ReflectError> {
    if world.get_resource::<FrameCounter>().is_some() {
        return Ok(false);
    }
    world.insert_resource(FrameCounter { value: 0 });
    Ok(true)
}

fn frame_counter_contains(world: &World) -> bool {
    world.get_resource::<FrameCounter>().is_some()
}

fn frame_counter_read_field(
    world: &World,
    field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    let resource = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    match field_name {
        "value" => Ok(ReflectedValue::Unsigned(resource.value as u64)),
        _ => Err(unknown_frame_counter_field(field_name)),
    }
}

fn frame_counter_read_fields(world: &World) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(vec![ReflectFieldValue::new(
        "value",
        frame_counter_read_field(world, "value")?,
    )])
}

fn frame_counter_write_field(
    world: &mut World,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, ReflectError> {
    let current = world
        .get_resource::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?;
    if field_name != "value" {
        return Err(unknown_frame_counter_field(field_name));
    }
    let next = expect_frame_counter_value(field_name, value)?;
    if current.value == next {
        return Ok(false);
    }

    world
        .get_resource_mut::<FrameCounter>()
        .ok_or_else(missing_frame_counter_resource)?
        .value = next;
    Ok(true)
}

fn expect_frame_counter_value(
    field_name: &str,
    value: ReflectedValue,
) -> Result<u32, ReflectError> {
    match value {
        ReflectedValue::Unsigned(value) if u32::try_from(value).is_ok() => Ok(value as u32),
        ReflectedValue::Unsigned(_) => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "u32 Unsigned".to_string(),
            actual: "Unsigned".to_string(),
        }),
        value => Err(ReflectError::TypeMismatch {
            type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
            field_name: field_name.to_string(),
            expected: "Unsigned".to_string(),
            actual: value.type_name().to_string(),
        }),
    }
}

fn missing_frame_counter_resource() -> ReflectError {
    ReflectError::MissingResource {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
    }
}

fn unknown_frame_counter_field(field_name: &str) -> ReflectError {
    ReflectError::UnknownField {
        type_path: FRAME_COUNTER_TYPE_PATH.to_string(),
        field_name: field_name.to_string(),
    }
}
