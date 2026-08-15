use crate::core::resource::{
    AssetReference, AssetUuid, MaterialMarker, ModelMarker, ResourceDiagnostic, ResourceEventKind,
    ResourceHandle, ResourceId, ResourceKind, ResourceLocator, ResourceLocatorError,
    ResourceManager, ResourceRecord, ResourceRegistry, ResourceRegistryError, ResourceScheme,
    ResourceState, RuntimeResourceState, UntypedResourceHandle,
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
    let locator = locator("res://materials/default.zmaterial");
    let reference = AssetReference::new(uuid, locator.clone());
    let json = serde_json::to_string(&reference).expect("serialize reference");
    let decoded: AssetReference = serde_json::from_str(&json).expect("deserialize reference");

    assert!(json.contains("\"url\""));
    assert!(!json.contains("\"locator\""));
    assert_eq!(decoded.uuid, uuid);
    assert_eq!(decoded.locator, locator);
    assert_eq!(uuid.to_string().parse::<AssetUuid>().unwrap(), uuid);
}

#[test]
fn resource_id_is_stable_for_asset_uuid() {
    let uuid = AssetUuid::from_stable_label("test://robot");
    let other_uuid = AssetUuid::from_stable_label("test://robot#mesh0");

    assert_eq!(
        ResourceId::from_asset_uuid(uuid),
        ResourceId::from_asset_uuid(uuid)
    );
    assert_ne!(
        ResourceId::from_asset_uuid(uuid),
        ResourceId::from_asset_uuid(other_uuid)
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
    let manager = ResourceManager::new();
    let original = record("res://materials/default.zmaterial", ResourceKind::Material);
    let id = original.id;
    manager
        .register_record(original.clone())
        .expect("register original record");

    let renamed = manager
        .rename(
            &original.primary_locator,
            locator("res://materials/default-renamed.zmaterial"),
        )
        .expect("rename should succeed");

    assert_eq!(renamed.id, id);
    assert!(manager
        .registry()
        .get_by_locator(&locator("res://materials/default.zmaterial"))
        .is_none());
    assert_eq!(
        manager
            .registry()
            .get_by_locator(&locator("res://materials/default-renamed.zmaterial"))
            .expect("renamed locator should exist")
            .id,
        id
    );

    let removed = manager
        .remove_by_locator(&locator("res://materials/default-renamed.zmaterial"))
        .expect("remove transaction should succeed")
        .expect("record should exist");
    assert_eq!(removed.id, id);
    assert!(manager.registry().get(id).is_none());
}

#[test]
fn registry_rename_reports_missing_locator_with_resource_error() {
    let manager = ResourceManager::new();
    let missing = locator("res://materials/missing.zmaterial");
    let target = locator("res://materials/target.zmaterial");

    let error = manager
        .rename(&missing, target)
        .expect_err("missing locator should return ResourceRegistryError");

    match error {
        ResourceRegistryError::MissingRecordForLocator { locator } => {
            assert_eq!(locator, missing.to_string());
        }
        other => panic!("expected missing resource locator ResourceRegistryError, got {other:?}"),
    }
}

#[test]
fn manager_failed_reload_keeps_last_good_payload_and_emits_events() {
    let manager = ResourceManager::new();
    let events = manager.subscribe();
    let locator = locator("res://models/cube.obj");
    let id = ResourceId::from_locator(&locator);
    let mut record = ResourceRecord::new(id, ResourceKind::Model, locator.clone());
    record.state = ResourceState::Pending;

    let handle = manager
        .register_ready(record, TestPayload { name: "cube-ready" })
        .expect("register ready model");
    let typed = handle.typed::<ModelMarker>().expect("model handle");

    let added = events.recv().expect("added event");
    assert_eq!(added.kind, ResourceEventKind::Added);
    assert_eq!(added.id, id);

    manager
        .start_reload(id, vec![ResourceDiagnostic::error("reload started")])
        .expect("start reload");
    let reloading = events.recv().expect("reload event");
    assert_eq!(reloading.kind, ResourceEventKind::Updated);

    manager
        .fail_reload(id, vec![ResourceDiagnostic::error("shader compile failed")])
        .expect("fail reload");
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
fn resource_state_rejects_error_to_ready_without_reloading() {
    let manager = ResourceManager::new();
    let locator = locator("res://models/broken.obj");
    let id = ResourceId::from_locator(&locator);
    manager
        .register_record(
            ResourceRecord::new(id, ResourceKind::Model, locator.clone())
                .with_state(ResourceState::Error)
                .with_diagnostics(vec![ResourceDiagnostic::error("initial import failed")]),
        )
        .expect("register failed record");
    let events = manager.subscribe();

    let error = manager
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Model, locator),
            TestPayload {
                name: "should-not-load",
            },
        )
        .expect_err("error to ready requires an explicit recovery operation");

    assert!(matches!(
        error,
        ResourceRegistryError::InvalidStateTransition { .. }
    ));
    assert!(events.try_recv().is_err());
    assert!(manager
        .get::<ModelMarker, TestPayload>(ResourceHandle::new(id))
        .is_none());
    let record = manager.registry().get(id).cloned().expect("record exists");
    assert_eq!(record.state, ResourceState::Error);
    assert_eq!(record.failure_reason(), Some("initial import failed"));
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Unloaded)
    );
}

#[test]
fn resource_state_recovers_from_error_only_through_reloading() {
    let manager = ResourceManager::new();
    let locator = locator("res://models/retry.obj");
    let id = ResourceId::from_locator(&locator);

    manager
        .register_record(
            ResourceRecord::new(id, ResourceKind::Model, locator.clone())
                .with_state(ResourceState::Error)
                .with_diagnostics(vec![ResourceDiagnostic::error("decode failed")]),
        )
        .expect("register failed record");
    let reloading = manager
        .start_reload(id, vec![ResourceDiagnostic::error("retry started")])
        .expect("error records can enter retry reload");
    assert_eq!(reloading.state, ResourceState::Reloading);

    let handle = manager
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Model, locator),
            TestPayload {
                name: "retry-ready",
            },
        )
        .expect("finish reload")
        .typed::<ModelMarker>()
        .expect("model handle");

    let payload = manager
        .get::<ModelMarker, TestPayload>(handle)
        .expect("retry payload");
    assert_eq!(payload.name, "retry-ready");
    let record = manager.registry().get(id).cloned().expect("record exists");
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(record.failure_reason(), None);
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
    );
}

#[test]
fn resource_state_rejects_reload_failure_without_reload_boundary() {
    let manager = ResourceManager::new();
    let locator = locator("res://models/ready.obj");
    let id = ResourceId::from_locator(&locator);
    manager
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Model, locator),
            TestPayload { name: "ready" },
        )
        .expect("register ready record");

    assert!(manager
        .fail_reload(id, vec![ResourceDiagnostic::error("unexpected failure")])
        .is_err());
    let record = manager.registry().get(id).cloned().expect("record exists");
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(record.failure_reason(), None);
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
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
        .expect("register leased model")
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
        .expect("register initial ready record")
        .typed::<ModelMarker>()
        .expect("typed model handle");
    let added = events.recv().expect("added event");
    assert_eq!(added.kind, ResourceEventKind::Added);

    manager
        .register_ready(record, TestPayload { name: "cube-ready" })
        .expect("register unchanged ready record");

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

#[test]
fn register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics() {
    let manager = ResourceManager::new();
    let locator = locator("res://materials/diagnostic.zmaterial");
    let id = ResourceId::from_locator(&locator);
    let diagnostic = ResourceDiagnostic::error("shader contract warning");
    let record = ResourceRecord::new(id, ResourceKind::Material, locator.clone())
        .with_diagnostics(vec![diagnostic.clone()]);

    manager
        .register_ready(record, TestPayload { name: "material" })
        .expect("register diagnostic material");

    assert_eq!(
        manager
            .registry()
            .get(id)
            .expect("record exists")
            .diagnostics,
        vec![diagnostic]
    );

    manager
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Material, locator),
            TestPayload { name: "material" },
        )
        .expect("register clean material");

    assert!(
        manager
            .registry()
            .get(id)
            .expect("record exists")
            .diagnostics
            .is_empty(),
        "a clean ready record must replace stale diagnostics"
    );
}

#[test]
fn register_ready_bumps_revision_when_dependency_ids_change() {
    let manager = ResourceManager::new();
    let events = manager.subscribe();
    let locator = locator("res://materials/grid.zmaterial");
    let id = ResourceId::from_locator(&locator);
    let dependency = ResourceId::from_stable_label("res://textures/checker.png");
    let record = ResourceRecord::new(id, ResourceKind::Material, locator);

    manager
        .register_ready(record.clone(), TestPayload { name: "material" })
        .expect("register material");
    let added = events.recv().expect("added event");
    assert_eq!(added.kind, ResourceEventKind::Added);

    let mut changed = record;
    changed.dependency_ids = vec![dependency];
    manager
        .register_ready(changed, TestPayload { name: "material" })
        .expect("register changed material");

    let updated = events.recv().expect("dependency update event");
    assert_eq!(updated.kind, ResourceEventKind::Updated);
    assert_eq!(updated.revision, 2);
    let registry = manager.registry();
    let stored = registry.get(id).expect("record exists");
    assert_eq!(stored.revision, 2);
    assert_eq!(stored.dependency_ids, vec![dependency]);
}

#[test]
fn resource_manager_hot_paths_avoid_redundant_record_projection() {
    let registry = include_str!("registry.rs");
    let payload_ops = include_str!("manager/payload_ops.rs");
    let lease_ops = include_str!("manager/lease_ops.rs");
    let commit = include_str!("manager/commit.rs");
    let registry_ops = include_str!("manager/registry_ops.rs");
    let registry_export = include_str!("manager/registry_export.rs");

    assert!(!registry.contains("self.by_id.get(&record.id).cloned()"));
    assert!(!registry.contains("pub fn upsert("));
    assert!(!registry.contains("pub fn remove_by_locator("));
    assert!(registry.contains("pub(crate) struct ResourceRegistryStaging"));
    assert!(!payload_ops.contains("registry.upsert(record.clone())"));
    assert!(!payload_ops.contains("self.snapshot::<TMarker, TData>(handle)"));
    assert!(lease_ops.contains("let mut authority = self.lock_authority_write();"));
    assert!(lease_ops.contains("slot.residency_token != residency_token"));
    assert!(!lease_ops.contains("lock_payloads_write"));
    assert!(commit.contains("Ok(self.prepare_commit(batch)?.commit())"));
    assert!(commit.contains("let commit_serial = self.lock_commit_serial();"));
    assert!(commit.contains("apply_staged(&mut authority, self.staged)"));
    assert!(commit.find("lock_commit_serial").unwrap() < commit.find("publish_event").unwrap());
    assert!(!registry_ops.contains("registry.upsert(record.clone())"));
    assert!(!registry_export.contains("left.id.to_string().cmp(&right.id.to_string())"));
}

#[test]
fn registry_staging_rejects_locator_collisions_without_partial_mutation() {
    let shared_locator = locator("res://materials/shared.zmat");
    let first_id = ResourceId::from_stable_label("registry-staging-first");
    let conflicting_id = ResourceId::from_stable_label("registry-staging-conflicting");
    let first = ResourceRecord::new(first_id, ResourceKind::Material, shared_locator.clone());
    let conflicting = ResourceRecord::new(
        conflicting_id,
        ResourceKind::Material,
        shared_locator.clone(),
    );
    let mut staging = ResourceRegistry::default().begin_staging();

    staging
        .stage_record(first.clone())
        .expect("first staged record should be accepted");
    let error = staging
        .stage_record(conflicting)
        .expect_err("occupied locator must reject a different resource id");

    assert!(matches!(
        error,
        ResourceRegistryError::LocatorOccupied { .. }
    ));
    let registry = staging.finish();
    assert_eq!(registry.get(first.id), Some(&first));
    assert!(registry.get(conflicting_id).is_none());
}

#[test]
fn registry_staging_preserves_resource_identity_after_staged_removal() {
    let original = ResourceRecord::new(
        ResourceId::from_stable_label("registry-staging-stable-identity"),
        ResourceKind::Model,
        locator("res://models/stable-identity.glb"),
    );
    let mut initial = ResourceRegistry::default().begin_staging();
    initial
        .stage_record(original.clone())
        .expect("initial staged record should be accepted");
    let registry = initial.finish();

    let mut kind_candidate = registry.begin_staging();
    kind_candidate.stage_remove_locator(&original.primary_locator);
    let kind_error = kind_candidate
        .stage_record(ResourceRecord::new(
            original.id,
            ResourceKind::Material,
            original.primary_locator.clone(),
        ))
        .expect_err("staged removal must not erase the resource kind identity");
    assert!(matches!(
        kind_error,
        ResourceRegistryError::KindConflict { .. }
    ));

    let mut locator_candidate = registry.begin_staging();
    locator_candidate.stage_remove_locator(&original.primary_locator);
    let relocated = locator("res://models/relocated-identity.glb");
    let locator_error = locator_candidate
        .stage_record(ResourceRecord::new(original.id, original.kind, relocated))
        .expect_err("staged removal must not bypass explicit rename semantics");
    assert!(matches!(
        locator_error,
        ResourceRegistryError::ExplicitRenameRequired { .. }
    ));

    assert_eq!(registry.get(original.id), Some(&original));
}
