use serde_json::json;
use zircon_runtime::scene::{
    json_from_reflected, reflected_from_json, DynamicScene, DynamicSceneError, NodeKind,
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot, World,
};
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::serialization::{LoadError, MigrateError};

const V0_DYNAMIC_SCENE_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tests/fixtures/serialization/scene-dynamic/v0/dynamic-scene.json"
));

#[test]
fn v0_tag_shape_stays_arbitrary_reflected_json() {
    let loaded = reflected_from_json(r#"{ "kind": "Entity", "value": 7 }"#).unwrap();
    assert_eq!(
        loaded.value,
        ReflectedValue::Json(json!({ "kind": "Entity", "value": 7 }))
    );
    let current = json_from_reflected(&loaded.value).unwrap();
    assert!(current.contains("zircon.scene.reflected-json"));
}

#[test]
fn v0_dynamic_fixture_resaves_byte_identically() {
    let legacy: serde_json::Value = serde_json::from_str(V0_DYNAMIC_SCENE_JSON).unwrap();
    assert!(legacy.get("$zircon").is_none());
    assert_eq!(legacy["format_version"], 1);

    let migrated = DynamicScene::from_versioned_json(V0_DYNAMIC_SCENE_JSON).unwrap();
    let first = migrated.to_versioned_json_pretty().unwrap();
    let current: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert!(current["$zircon"]["payload"]
        .get("format_version")
        .is_none());
    let reloaded = DynamicScene::from_versioned_json(&first).unwrap();
    let second = reloaded.to_versioned_json_pretty().unwrap();
    assert_eq!(second.as_bytes(), first.as_bytes());
}

#[test]
fn project_world_v0_migrates_without_a_legacy_dto() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.rename_node(entity, "Legacy Mesh").unwrap();
    let legacy = serde_json::to_string(&json!({
        "format_version": 2,
        "world": world
    }))
    .unwrap();

    let scene = DynamicScene::from_versioned_json(&legacy).unwrap();
    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].record.name, "Legacy Mesh");
    assert_eq!(scene.entities[0].record.kind, NodeKind::Mesh);
    let current: serde_json::Value =
        serde_json::from_str(&scene.to_versioned_json_pretty().unwrap()).unwrap();
    assert!(current["$zircon"]["payload"]
        .get("format_version")
        .is_none());
}

#[test]
fn archive_embeds_scene_envelope_without_inner_version() {
    let slot = RuntimeSessionSlot::from_world("slot", &World::empty()).unwrap();
    assert_eq!(slot.summary().scene_format_version, 2);
    let archive = RuntimeSessionArchive::from_slots(vec![slot]).unwrap();
    let text = archive.to_versioned_json_pretty().unwrap();
    let document: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        document["slots"][0]["scene"]["$zircon"]["header"]["schema_version"],
        2
    );
    assert!(document["slots"][0]["scene"]["$zircon"]["payload"]
        .get("format_version")
        .is_none());
}

#[test]
fn current_envelope_rejects_retired_inner_version_field() {
    let mut current: serde_json::Value =
        serde_json::from_str(&DynamicScene::empty().to_versioned_json_pretty().unwrap()).unwrap();
    current["$zircon"]["payload"]["format_version"] = json!(1);

    let error = DynamicScene::from_versioned_json(&current.to_string()).unwrap_err();
    assert!(matches!(
        error,
        DynamicSceneError::SerializationLoad(source)
            if matches!(source.as_ref(), LoadError::PayloadDecode { .. })
    ));
}

#[test]
fn v1_envelope_migrates_to_versionless_v2_payload() {
    let mut v1: serde_json::Value =
        serde_json::from_str(&DynamicScene::empty().to_versioned_json_pretty().unwrap()).unwrap();
    v1["$zircon"]["header"]["schema_version"] = json!(1);
    v1["$zircon"]["payload"]["format_version"] = json!(1);

    let migrated = DynamicScene::from_versioned_json(&v1.to_string()).unwrap();
    let current: serde_json::Value =
        serde_json::from_str(&migrated.to_versioned_json_pretty().unwrap()).unwrap();
    assert_eq!(current["$zircon"]["header"]["schema_version"], 2);
    assert!(current["$zircon"]["payload"]
        .get("format_version")
        .is_none());
}

#[test]
fn v1_envelope_requires_the_historical_inner_version_contract() {
    for retired_inner_version in [None, Some(json!(9))] {
        let mut v1: serde_json::Value =
            serde_json::from_str(&DynamicScene::empty().to_versioned_json_pretty().unwrap())
                .unwrap();
        v1["$zircon"]["header"]["schema_version"] = json!(1);
        match retired_inner_version {
            Some(version) => v1["$zircon"]["payload"]["format_version"] = version,
            None => {
                v1["$zircon"]["payload"]
                    .as_object_mut()
                    .unwrap()
                    .remove("format_version");
            }
        }

        let error = DynamicScene::from_versioned_json(&v1.to_string())
            .expect_err("v1 payloads must preserve their historical inner version contract");
        assert!(matches!(
            error,
            DynamicSceneError::SerializationLoad(source)
                if matches!(
                    source.as_ref(),
                    LoadError::Migration(MigrateError::StepFailed {
                        from_version: 1,
                        ..
                    })
                )
        ));
    }
}

#[test]
fn archive_rejects_future_embedded_header_before_payload_decode() {
    let slot = RuntimeSessionSlot::from_world("slot", &World::empty()).unwrap();
    let archive = RuntimeSessionArchive::from_slots(vec![slot]).unwrap();
    let mut document: serde_json::Value =
        serde_json::from_str(&archive.to_versioned_json_pretty().unwrap()).unwrap();
    document["slots"][0]["scene"]["$zircon"]["header"]["schema_version"] = json!(3);
    document["slots"][0]["scene"]["$zircon"]["payload"] = json!({ "invalid": true });

    let error = RuntimeSessionArchive::from_versioned_json(&document.to_string()).unwrap_err();
    assert!(matches!(
        error,
        RuntimeSessionArchiveError::DynamicScene(DynamicSceneError::SerializationLoad(source))
            if matches!(source.as_ref(), LoadError::FutureVersion { found: 3, supported: 2, .. })
    ));
}
