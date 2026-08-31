use crate::{
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
    assert!(
        manager
            .registry()
            .get_by_locator(&locator("res://materials/default.zmaterial"))
            .is_none()
    );
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
fn registry_staging_explicit_rename_preserves_identity_for_followup_record() {
    let original_locator = locator("res://models/staging-original.glb");
    let relocated_locator = locator("res://models/staging-relocated.glb");
    let id = ResourceId::from_stable_label("staging-relocation-model");
    let original = ResourceRecord::new(id, ResourceKind::Model, original_locator.clone());
    let mut registry = ResourceRegistry::default();
    registry.insert_unchecked(original);
    let mut staging = registry.begin_staging();

    staging
        .stage_rename_locator(&original_locator, relocated_locator.clone())
        .expect("explicit staging rename should preserve the record identity");
    let mut refreshed = staging
        .get(id)
        .cloned()
        .expect("renamed record should remain available by stable id");
    refreshed.source_hash = "reimported-after-relocation".to_owned();
    staging
        .stage_record(refreshed)
        .expect("a renamed record should accept a followup staging update");

    let registry = staging.finish();
    assert!(registry.get_by_locator(&original_locator).is_none());
    let relocated = registry
        .get_by_locator(&relocated_locator)
        .expect("relocated locator should resolve after staging finishes");
    assert_eq!(relocated.id, id);
    assert_eq!(relocated.source_hash, "reimported-after-relocation");
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
    assert!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(id))
            .is_none()
    );
    let record = manager.registry().get(id).cloned().expect("record exists");
    assert_eq!(record.state, ResourceState::Error);
    assert_eq!(record.failure_reason(), Some("initial import failed"));
    assert_eq!(manager.runtime_state(id), Some(RuntimeResourceState::Error));
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

    assert!(
        manager
            .fail_reload(id, vec![ResourceDiagnostic::error("unexpected failure")])
            .is_err()
    );
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
    let crate_root = include_str!("lib.rs");
    let assembly = include_str!("assembly.rs");
    let payload_ops = include_str!("manager/payload_ops.rs");
    let lease_ops = include_str!("manager/lease_ops.rs");
    let commit = include_str!("manager/commit.rs");
    let resource_manager = include_str!("manager/resource_manager.rs");
    let runtime_slot = include_str!("manager/runtime_slot.rs");
    let lease = include_str!("lease.rs");
    let registry_ops = include_str!("manager/registry_ops.rs");
    let registry_export = include_str!("manager/registry_export.rs");

    assert!(!registry.contains("self.by_id.get(&record.id).cloned()"));
    assert!(!registry.contains("pub fn upsert("));
    assert!(!registry.contains("pub fn remove_by_locator("));
    assert!(registry.contains("pub struct ResourceRegistryStaging"));
    assert!(!crate_root.contains("pub use registry::ResourceRegistryStaging"));
    assert!(assembly.contains("pub use crate::registry::ResourceRegistryStaging;"));
    assert!(!payload_ops.contains("registry.upsert(record.clone())"));
    assert!(!payload_ops.contains("self.snapshot::<TMarker, TData>(handle)"));
    assert!(lease_ops.contains("let mut authority = self.lock_authority_write();"));
    assert!(lease_ops.contains("Arc::ptr_eq(&slot.lease_identity, &lease_identity)"));
    assert!(lease_ops.contains("drop(lease_identity);"));
    assert!(lease_ops.contains("Arc::strong_count(&slot.lease_identity) == 1"));
    assert!(!runtime_slot.contains("ref_count:"));
    assert!(!lease_ops.contains("slot.ref_count"));
    for source in [lease, lease_ops, resource_manager, runtime_slot, commit] {
        assert!(!source.contains("residency_token"));
        assert!(!source.contains("next_residency_token"));
    }
    assert!(!lease_ops.contains("lock_payloads_write"));
    assert!(commit.contains("Ok(self.prepare_commit(batch)?.commit())"));
    assert!(commit.contains("let commit_serial = self.lock_commit_serial();"));
    assert!(commit.contains("apply_staged(&mut authority, self.staged, self.events.len())"));
    let prepared_commit = commit
        .split("impl PreparedResourceMutation<'_>")
        .nth(1)
        .and_then(|source| source.split("impl ResourceManager").next())
        .expect("prepared Resource commit implementation");
    assert!(
        prepared_commit.find("publish_permitted").unwrap()
            < prepared_commit.find("drop(self.commit_serial)").unwrap()
    );
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

#[test]
fn registry_staging_shares_unchanged_record_storage() {
    const RECORD_COUNT: usize = 4_096;

    let mut initial = ResourceRegistry::default().begin_staging();
    for index in 0..RECORD_COUNT {
        let locator = locator(&format!("res://models/staging-{index:04}.glb"));
        initial
            .stage_record(ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Model,
                locator,
            ))
            .expect("initial staged record should be accepted");
    }
    let registry = initial.finish();
    let unchanged_id = ResourceId::from_locator(&locator("res://models/staging-2048.glb"));
    let original_path = registry
        .get(unchanged_id)
        .expect("unchanged record should exist")
        .primary_locator
        .path()
        .as_ptr();

    let changed_id = ResourceId::from_locator(&locator("res://models/staging-0000.glb"));
    let mut staging = registry.begin_staging();
    staging
        .stage_record(
            registry
                .get(changed_id)
                .expect("changed record should exist")
                .clone(),
        )
        .expect("replacing one staged record should succeed");
    let staged_path = staging
        .get(unchanged_id)
        .expect("staged registry should expose unchanged record")
        .primary_locator
        .path()
        .as_ptr();

    assert_eq!(
        staged_path, original_path,
        "begin_staging must share immutable record storage instead of deep-cloning the registry"
    );
}

#[test]
#[ignore = "release performance gate"]
fn registry_staging_copy_on_write_release_gate() {
    use std::collections::HashMap;
    use std::hint::black_box;
    use std::time::Instant;

    const RECORD_COUNT: usize = 4_096;
    const ROUNDS_PER_SAMPLE: usize = 16;
    const SAMPLE_PAIRS: usize = 21;

    fn legacy_begin_staging(registry: &ResourceRegistry) {
        let mut copied_registry = ResourceRegistry::default();
        let mut identities = HashMap::with_capacity(registry.values().count());
        for record in registry.values() {
            copied_registry.insert_unchecked(record.clone());
            identities.insert(record.id, (record.kind, record.primary_locator.clone()));
        }
        black_box((copied_registry, identities));
    }

    fn optimized_begin_staging(registry: &ResourceRegistry, probe_id: ResourceId) {
        let staging = registry.begin_staging();
        black_box(
            staging
                .get(probe_id)
                .expect("optimized staging should expose the probe record")
                .revision,
        );
    }

    fn measure(mut operation: impl FnMut()) -> u128 {
        let started = Instant::now();
        for _ in 0..ROUNDS_PER_SAMPLE {
            operation();
        }
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    let mut initial = ResourceRegistry::default().begin_staging();
    for index in 0..RECORD_COUNT {
        let locator = locator(&format!("res://textures/staging-bench-{index:04}.png"));
        initial
            .stage_record(ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Texture,
                locator,
            ))
            .expect("benchmark fixture record should be accepted");
    }
    let registry = initial.finish();
    let probe_id = ResourceId::from_locator(&locator("res://textures/staging-bench-2048.png"));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| legacy_begin_staging(black_box(&registry))));
            optimized_samples.push(measure(|| {
                optimized_begin_staging(black_box(&registry), probe_id)
            }));
        } else {
            optimized_samples.push(measure(|| {
                optimized_begin_staging(black_box(&registry), probe_id)
            }));
            legacy_samples.push(measure(|| legacy_begin_staging(black_box(&registry))));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    let reduction_pct = 100.0 * (1.0 - optimized_p95_ns as f64 / legacy_p95_ns as f64);
    let legacy_samples_ns = legacy_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let optimized_samples_ns = optimized_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");

    println!(
        "PERF-MVP-RAR-P2-002 sample_pairs={SAMPLE_PAIRS} registry_records={RECORD_COUNT} rounds_per_sample={ROUNDS_PER_SAMPLE} legacy_samples_ns={legacy_samples_ns} optimized_samples_ns={optimized_samples_ns} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} reduction_pct={reduction_pct:.3} legacy_record_clones_per_begin={RECORD_COUNT} optimized_record_clones_per_begin=0 legacy_locator_clones_per_begin={} optimized_locator_clones_per_begin=0",
        RECORD_COUNT * 3
    );
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "copy-on-write staging must reduce nearest-rank P95 by at least 75%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
