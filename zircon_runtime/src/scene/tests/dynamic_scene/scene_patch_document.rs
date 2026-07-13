use super::*;
use zircon_runtime_interface::serialization::LoadError;

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

    let encoded = DynamicScene::from_world(&source)
        .expect("source world should export")
        .to_versioned_json_pretty()
        .expect("dynamic scene should serialize");
    assert!(encoded.contains("\"format_version\": 1"));
    assert!(encoded.contains("\"schema_id\": \"zircon.scene.dynamic-scene\""));
    assert!(encoded.contains(CLOUD_LAYER_TYPE_PATH));
    assert_text_excludes_authoring_tokens(
        "dynamic scene JSON",
        &encoded,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let scene = DynamicScene::from_versioned_json(&encoded)
        .expect("versioned dynamic scene should deserialize");

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
    let expected_component_instance_count = patch
        .scene
        .entities
        .iter()
        .map(|entity| entity.components.len())
        .sum::<usize>();
    assert_eq!(
        patch
            .scene
            .entities
            .iter()
            .flat_map(|entity| &entity.components)
            .filter(|component| component.plugin_owned)
            .count(),
        1
    );

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
    assert_eq!(
        preview.component_instance_count,
        expected_component_instance_count
    );
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
    let descriptor = cloud_layer_descriptor();
    scene.component_types.push(descriptor.clone());
    let registration = crate::scene::reflect::registration_from_component_descriptor(&descriptor)
        .expect("cloud layer descriptor should produce valid reflection metadata");
    let mut world = World::empty();
    world
        .type_registry_mut_for_tests()
        .register(crate::scene::reflect::RuntimeTypeRegistration::metadata(
            registration,
        ))
        .expect("metadata-only duplicate fixture should register");

    let error = scene
        .spawn_into(&mut world)
        .expect_err("duplicate runtime registration should preserve scene error source");

    assert!(matches!(
        error,
        DynamicSceneError::WorldMutation(SceneError::Reflect(
            ReflectError::DuplicateTypePath { type_path }
        )) if type_path == CLOUD_LAYER_TYPE_PATH
    ));
}

#[test]
fn dynamic_scene_rejects_future_envelope_header_before_payload_decode() {
    let scene = DynamicScene::empty();
    let current = scene
        .to_versioned_json_pretty()
        .expect("current dynamic scene should serialize");
    let mut future: serde_json::Value = serde_json::from_str(&current).unwrap();
    future["$zircon"]["header"]["schema_version"] = serde_json::Value::from(2);
    future["$zircon"]["payload"] = json!({ "not": "a dynamic scene" });

    let error = DynamicScene::from_versioned_json(&future.to_string())
        .expect_err("future scene versions require a newer reader");
    assert!(matches!(
        error,
        DynamicSceneError::SerializationLoad(source)
            if matches!(
                source.as_ref(),
                LoadError::FutureVersion {
                    found: 2,
                    supported: 1,
                    ..
                }
            )
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
