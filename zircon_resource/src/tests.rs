use crate::{
    AssetReference, AssetUuid, MaterialMarker, ModelMarker, ResourceDiagnostic, ResourceEventKind,
    ResourceHandle, ResourceId, ResourceKind, ResourceLocator, ResourceLocatorError,
    ResourceManager, ResourceRecord, ResourceScheme, ResourceState, RuntimeResourceState,
    UntypedResourceHandle,
};

#[derive(Debug, PartialEq, Eq)]
struct TestPayload {
    name: &'static str,
}

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).expect("valid locator")
}

fn record(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = locator(locator_text);
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

#[test]
fn locator_normalizes_scheme_path_and_label() {
    let locator = ResourceLocator::parse("res://textures\\material/../brick.png#albedo")
        .expect("locator should parse");

    assert_eq!(locator.scheme(), ResourceScheme::Res);
    assert_eq!(locator.path(), "textures/brick.png");
    assert_eq!(locator.label(), Some("albedo"));
    assert_eq!(locator.to_string(), "res://textures/brick.png#albedo");
}

#[test]
fn locator_rejects_escape_attempts() {
    let error = ResourceLocator::parse("lib://../../outside.bin").expect_err("must reject");
    assert_eq!(
        error,
        ResourceLocatorError::EscapeAttempt("../../outside.bin".to_string())
    );
}

#[test]
fn resource_ids_are_stable_for_persistent_schemes_and_not_for_mem() {
    let res_locator = locator("res://models/ship.glb#mesh0");
    let builtin_locator = locator("builtin://meshes/cube");
    let mem_locator = locator("mem://viewport/selection");

    assert_eq!(
        ResourceId::from_locator(&res_locator),
        ResourceId::from_locator(&res_locator)
    );
    assert_eq!(
        ResourceId::from_locator(&builtin_locator),
        ResourceId::from_locator(&builtin_locator)
    );
    assert_ne!(
        ResourceId::from_locator(&mem_locator),
        ResourceId::from_locator(&mem_locator)
    );
}

#[test]
fn asset_uuid_and_reference_roundtrip() {
    let uuid = AssetUuid::new();
    let locator = locator("res://materials/default.material.toml");
    let reference = AssetReference::new(uuid, locator.clone());
    let json = serde_json::to_string(&reference).expect("serialize reference");
    let decoded: AssetReference = serde_json::from_str(&json).expect("deserialize reference");

    assert_eq!(decoded.uuid, uuid);
    assert_eq!(decoded.locator, locator);
    assert_eq!(uuid.to_string().parse::<AssetUuid>().unwrap(), uuid);
}

#[test]
fn resource_id_is_stable_for_asset_uuid_and_label() {
    let uuid = AssetUuid::from_stable_label("test://robot");

    assert_eq!(
        ResourceId::from_asset_uuid_label(uuid, None),
        ResourceId::from_asset_uuid_label(uuid, None)
    );
    assert_eq!(
        ResourceId::from_asset_uuid_label(uuid, Some("mesh0")),
        ResourceId::from_asset_uuid_label(uuid, Some("mesh0"))
    );
    assert_ne!(
        ResourceId::from_asset_uuid_label(uuid, None),
        ResourceId::from_asset_uuid_label(uuid, Some("mesh0"))
    );
}

#[test]
fn resource_id_display_roundtrips_through_parse() {
    let locator = locator("res://models/robot.glb#mesh0");
    let id = ResourceId::from_locator(&locator);
    let text = id.to_string();

    assert_eq!(text.parse::<ResourceId>().unwrap(), id);
    assert!("not-a-resource-id".parse::<ResourceId>().is_err());
}

#[test]
fn typed_and_untyped_handles_roundtrip() {
    let id = ResourceId::from_stable_label("res://models/robot.glb");
    let typed = ResourceHandle::<ModelMarker>::new(id);
    let untyped: UntypedResourceHandle = typed.into();
    let typed_back = untyped.typed::<ModelMarker>().expect("kind should match");

    assert_eq!(typed.id(), id);
    assert_eq!(untyped.id(), id);
    assert_eq!(typed_back.id(), id);
    assert!(untyped.typed::<MaterialMarker>().is_none());
}

#[test]
fn registry_rename_preserves_id_and_remove_clears_lookup() {
    let mut registry = crate::ResourceRegistry::default();
    let original = record(
        "res://materials/default.material.toml",
        ResourceKind::Material,
    );
    let id = original.id;
    registry.upsert(original.clone());

    let renamed = registry
        .rename(
            &original.primary_locator,
            locator("res://materials/default-renamed.material.toml"),
        )
        .expect("rename should succeed");

    assert_eq!(renamed.id, id);
    assert!(registry
        .get_by_locator(&locator("res://materials/default.material.toml"))
        .is_none());
    assert_eq!(
        registry
            .get_by_locator(&locator("res://materials/default-renamed.material.toml"))
            .expect("renamed locator should exist")
            .id,
        id
    );

    let removed = registry
        .remove_by_locator(&locator("res://materials/default-renamed.material.toml"))
        .expect("remove should succeed");
    assert_eq!(removed.id, id);
    assert!(registry.get(id).is_none());
}

#[test]
fn manager_failed_reload_keeps_last_good_payload_and_emits_events() {
    let manager = ResourceManager::new();
    let events = manager.subscribe();
    let locator = locator("res://models/cube.obj");
    let id = ResourceId::from_locator(&locator);
    let mut record = ResourceRecord::new(id, ResourceKind::Model, locator.clone());
    record.state = ResourceState::Pending;

    let handle = manager.register_ready(record, TestPayload { name: "cube-ready" });
    let typed = handle.typed::<ModelMarker>().expect("model handle");

    let added = events.recv().expect("added event");
    assert_eq!(added.kind, ResourceEventKind::Added);
    assert_eq!(added.id, id);

    manager.start_reload(id, vec![ResourceDiagnostic::error("reload started")]);
    let reloading = events.recv().expect("reload event");
    assert_eq!(reloading.kind, ResourceEventKind::Updated);

    manager.fail_reload(id, vec![ResourceDiagnostic::error("shader compile failed")]);
    let failed = events.recv().expect("reload failed event");
    assert_eq!(failed.kind, ResourceEventKind::ReloadFailed);

    let payload = manager
        .get::<ModelMarker, TestPayload>(typed)
        .expect("last good payload");
    assert_eq!(payload.name, "cube-ready");

    let record = manager.registry().get(id).cloned().expect("record exists");
    assert_eq!(record.state, ResourceState::Error);
    assert_eq!(record.revision, 1);
    assert_eq!(
        record.diagnostics,
        vec![ResourceDiagnostic::error("shader compile failed")]
    );
}

#[test]
fn resource_leases_increment_refcount_and_drop_unloads_payload() {
    let manager = ResourceManager::new();
    let locator = locator("res://models/cube.obj");
    let id = ResourceId::from_locator(&locator);
    let handle = manager
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Model, locator),
            TestPayload {
                name: "leased-model",
            },
        )
        .typed::<ModelMarker>()
        .expect("typed model handle");

    assert_eq!(manager.ref_count(id), Some(0));
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
    );

    let lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("resource lease");
    assert_eq!(lease.name, "leased-model");
    assert_eq!(manager.ref_count(id), Some(1));
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
    );

    drop(lease);

    assert_eq!(manager.ref_count(id), Some(0));
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Unloaded)
    );
    assert!(manager.get::<ModelMarker, TestPayload>(handle).is_none());
}

#[test]
fn register_ready_is_idempotent_for_unchanged_records() {
    let manager = ResourceManager::new();
    let events = manager.subscribe();
    let locator = locator("res://models/cube.obj");
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Model, locator);

    let handle = manager
        .register_ready(record.clone(), TestPayload { name: "cube-ready" })
        .typed::<ModelMarker>()
        .expect("typed model handle");
    let added = events.recv().expect("added event");
    assert_eq!(added.kind, ResourceEventKind::Added);

    manager.register_ready(record, TestPayload { name: "cube-ready" });

    assert!(
        events.try_recv().is_err(),
        "unchanged ready registration must be a no-op"
    );
    assert_eq!(
        manager.registry().get(id).expect("record exists").revision,
        1,
        "unchanged ready registration must not bump revision"
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(handle)
            .expect("payload should remain resident")
            .name,
        "cube-ready"
    );
}
