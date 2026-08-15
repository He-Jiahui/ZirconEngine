use std::sync::Arc;

use crate::core::resource::{
    ModelMarker, ResourceDiagnostic, ResourceEventKind, ResourceHandle, ResourceId, ResourceKind,
    ResourceLocator, ResourceManager, ResourceMutationBatch, ResourceRecord, ResourceRegistryError,
    ResourceState, RuntimeResourceState,
};

#[derive(Debug, PartialEq, Eq)]
struct TestPayload(&'static str);

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).expect("valid resource locator")
}

fn record(id_label: &str, locator_text: &str) -> ResourceRecord {
    ResourceRecord::new(
        ResourceId::from_stable_label(id_label),
        ResourceKind::Model,
        locator(locator_text),
    )
}

fn ready_batch(record: ResourceRecord, name: &'static str) -> ResourceMutationBatch {
    ResourceMutationBatch::new().upsert_ready(record, TestPayload(name))
}

#[test]
fn add_then_remove_in_one_batch_is_a_generation_stable_noop() {
    let manager = ResourceManager::new();
    let transient = record("transient-model", "res://models/transient.glb");
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();

    let receipt = manager
        .commit(
            ready_batch(transient.clone(), "transient").remove(transient.primary_locator.clone()),
        )
        .expect("a net-zero batch should commit as a no-op");

    assert!(manager.registry().get(transient.id).is_none());
    assert!(receipt.handle(transient.id).is_none());
    assert!(receipt.removed(transient.id).is_none());
    assert_eq!(receipt.published_event_count(), 0);
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
}

#[test]
fn locator_collision_rejects_the_whole_batch_without_publication() {
    let manager = ResourceManager::new();
    let original = record("model-a", "res://models/shared.glb");
    manager
        .commit(ready_batch(original.clone(), "original"))
        .expect("initial registration");
    let events = manager.subscribe();
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();

    let conflicting = record("model-b", "res://models/shared.glb");
    let error = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(record("model-c", "res://models/independent.glb"))
                .upsert_ready(conflicting.clone(), TestPayload("conflicting")),
        )
        .expect_err("occupied locator must reject the transaction");

    assert_eq!(
        error,
        ResourceRegistryError::LocatorOccupied {
            locator: conflicting.primary_locator.to_string(),
            existing_id: original.id.to_string(),
            requested_id: conflicting.id.to_string(),
        }
    );
    assert!(manager
        .registry()
        .get(ResourceId::from_stable_label("model-c"))
        .is_none());
    assert_eq!(
        manager
            .registry()
            .get_by_locator(&original.primary_locator)
            .expect("original locator remains authoritative")
            .id,
        original.id
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(original.id))
            .expect("original payload remains resident")
            .0,
        "original"
    );
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
    assert!(events.try_recv().is_err());
}

#[test]
fn same_id_locator_change_requires_an_explicit_rename_operation() {
    let manager = ResourceManager::new();
    let original = record("rename-model", "res://models/original.glb");
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(original.clone()))
        .expect("initial record");
    let events = manager.subscribe();
    let renamed_locator = locator("res://models/renamed.glb");
    let mut implicit = original.clone();
    implicit.primary_locator = renamed_locator.clone();

    let error = manager
        .commit(ResourceMutationBatch::new().upsert_lazy(implicit))
        .expect_err("upsert must not bypass rename semantics");
    assert_eq!(
        error,
        ResourceRegistryError::ExplicitRenameRequired {
            id: original.id.to_string(),
            current_locator: original.primary_locator.to_string(),
            requested_locator: renamed_locator.to_string(),
        }
    );

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .rename(original.primary_locator.clone(), renamed_locator.clone()),
        )
        .expect("explicit rename");
    assert_eq!(
        receipt.record(original.id).unwrap().primary_locator,
        renamed_locator
    );
    let event = events.recv().expect("rename event");
    assert_eq!(event.kind, ResourceEventKind::Renamed);
    assert_eq!(event.previous_locator, Some(original.primary_locator));
}

#[test]
fn remove_then_same_id_kind_change_is_rejected_without_mutation() {
    let manager = ResourceManager::new();
    let original = record("stable-kind-model", "res://models/stable-kind.glb");
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(original.clone()))
        .expect("initial record");
    let events = manager.subscribe();
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();
    let replacement = ResourceRecord::new(
        original.id,
        ResourceKind::Material,
        original.primary_locator.clone(),
    );

    let error = manager
        .commit(
            ResourceMutationBatch::new()
                .remove(original.primary_locator.clone())
                .upsert_lazy(replacement),
        )
        .expect_err("remove must not erase the batch-local resource identity");

    assert_eq!(
        error,
        ResourceRegistryError::KindConflict {
            id: original.id.to_string(),
            current_kind: ResourceKind::Model,
            requested_kind: ResourceKind::Material,
        }
    );
    assert_eq!(manager.registry().get(original.id), Some(&original));
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
    assert!(events.try_recv().is_err());
}

#[test]
fn remove_then_same_id_locator_change_still_requires_explicit_rename() {
    let manager = ResourceManager::new();
    let original = record("stable-locator-model", "res://models/stable-locator.glb");
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(original.clone()))
        .expect("initial record");
    let relocated = locator("res://models/relocated.glb");
    let mut replacement = original.clone();
    replacement.primary_locator = relocated.clone();

    let error = manager
        .commit(
            ResourceMutationBatch::new()
                .remove(original.primary_locator.clone())
                .upsert_lazy(replacement),
        )
        .expect_err("remove must not bypass explicit rename semantics");

    assert_eq!(
        error,
        ResourceRegistryError::ExplicitRenameRequired {
            id: original.id.to_string(),
            current_locator: original.primary_locator.to_string(),
            requested_locator: relocated.to_string(),
        }
    );
    assert_eq!(manager.registry().get(original.id), Some(&original));
}

#[test]
fn explicit_rename_authorization_survives_remove_and_readd_in_one_batch() {
    let manager = ResourceManager::new();
    let original = record("authorized-relocation", "res://models/authorized-old.glb");
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(original.clone()))
        .expect("initial record");
    let events = manager.subscribe();
    let relocated = locator("res://models/authorized-new.glb");
    let mut replacement = original.clone();
    replacement.primary_locator = relocated.clone();

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .rename(original.primary_locator.clone(), relocated.clone())
                .remove(relocated.clone())
                .upsert_lazy(replacement),
        )
        .expect("an explicit rename authorizes the final locator");

    assert_eq!(
        receipt.record(original.id).unwrap().primary_locator,
        relocated
    );
    let event = events.recv().expect("net locator change event");
    assert_eq!(event.kind, ResourceEventKind::Renamed);
    assert_eq!(event.previous_locator, Some(original.primary_locator));
}

#[test]
fn one_batch_publishes_each_projection_once() {
    let manager = ResourceManager::new();
    let first = record("batch-a", "res://models/a.glb");
    let second = record("batch-b", "res://models/b.glb");
    let third = record("batch-c", "res://models/c.glb");

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(first.clone())
                .upsert_lazy(second.clone())
                .upsert_ready(third.clone(), TestPayload("ready")),
        )
        .expect("atomic batch");

    assert_eq!(receipt.management_generation(), 1);
    assert_eq!(receipt.readiness_generation(), 1);
    assert_eq!(receipt.published_event_count(), 3);
    assert_eq!(manager.management_generation().sequence(), 1);
    assert_eq!(manager.readiness_generation().sequence(), 1);
    assert_eq!(manager.management_generation().summary().total_count(), 3);
}

#[test]
fn payload_install_rejects_a_stale_record_revision() {
    let manager = ResourceManager::new();
    let original = record("revision-model", "res://models/revision.glb")
        .with_state(ResourceState::Ready)
        .with_source_hash("v1");
    let id = original.id;
    manager
        .commit(ready_batch(original.clone(), "v1"))
        .expect("initial ready record");
    let expected_revision = manager.registry().get(id).unwrap().revision;

    let changed = original.with_source_hash("v2");
    manager
        .commit(ready_batch(changed, "v2"))
        .expect("newer ready record");
    let actual_revision = manager.registry().get(id).unwrap().revision;
    let error = manager
        .commit(ResourceMutationBatch::new().store_payload(
            id,
            expected_revision,
            TestPayload("stale"),
        ))
        .expect_err("stale artifact payload must not publish");

    assert_eq!(
        error,
        ResourceRegistryError::RevisionConflict {
            id: id.to_string(),
            expected_revision,
            actual_revision: Some(actual_revision),
        }
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(id))
            .unwrap()
            .0,
        "v2"
    );
}

#[test]
fn importer_recovery_is_explicit_and_publishes_once() {
    let manager = ResourceManager::new();
    let failed =
        record("recover-model", "res://models/recover.glb").with_state(ResourceState::Error);
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(failed.clone()))
        .expect("failed record");
    let events = manager.subscribe();

    let ordinary_error = manager
        .commit(ready_batch(failed.clone(), "ordinary"))
        .expect_err("ordinary ready upsert cannot skip reload recovery");
    assert!(matches!(
        ordinary_error,
        ResourceRegistryError::InvalidStateTransition { .. }
    ));

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_imported_erased(failed.clone(), Arc::new(TestPayload("recovered"))),
        )
        .expect("explicit importer recovery");
    assert_eq!(receipt.published_event_count(), 1);
    assert_eq!(
        manager.registry().get(failed.id).unwrap().state,
        ResourceState::Ready
    );
    assert_eq!(events.recv().unwrap().kind, ResourceEventKind::Updated);
}

#[test]
fn releasing_the_last_lease_during_failed_reload_keeps_the_last_good_payload() {
    let manager = ResourceManager::new();
    let original = record("reload-lease-model", "res://models/reload-lease.glb");
    manager
        .commit(ready_batch(original.clone(), "last-good"))
        .expect("initial ready payload");
    let handle = ResourceHandle::<ModelMarker>::new(original.id);
    let lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("lease the last-good payload");

    manager
        .commit(ResourceMutationBatch::new().start_reload(
            original.id,
            vec![ResourceDiagnostic::error("reload started")],
        ))
        .expect("start reload");
    manager
        .commit(ResourceMutationBatch::new().fail_reload(
            original.id,
            vec![ResourceDiagnostic::error("reload failed")],
        ))
        .expect("record reload failure");
    drop(lease);

    assert_eq!(manager.ref_count(original.id), Some(0));
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Error)
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(handle)
            .expect("failed reload keeps the last-good payload")
            .0,
        "last-good"
    );

    let failed_lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("last-good payload remains leasable for fallback rendering");
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Error)
    );
    drop(failed_lease);
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Error)
    );
    assert!(manager.get_untyped(original.id).is_some());
}

#[test]
fn catalog_diagnostics_refresh_after_failed_reload_keeps_the_last_good_payload() {
    let manager = ResourceManager::new();
    let original = record(
        "reload-catalog-refresh-model",
        "res://models/reload-catalog-refresh.glb",
    );
    manager
        .commit(ready_batch(original.clone(), "last-good"))
        .expect("initial ready payload");
    manager
        .commit(ResourceMutationBatch::new().start_reload(
            original.id,
            vec![ResourceDiagnostic::error("reload started")],
        ))
        .expect("start reload");
    manager
        .commit(ResourceMutationBatch::new().fail_reload(
            original.id,
            vec![ResourceDiagnostic::error("reload failed")],
        ))
        .expect("record reload failure");
    let mut refreshed = manager
        .registry()
        .get(original.id)
        .cloned()
        .expect("failed record remains registered");
    refreshed.diagnostics = vec![ResourceDiagnostic::error("catalog confirmed failure")];

    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(refreshed))
        .expect("catalog diagnostics refresh");

    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Error)
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(original.id))
            .expect("catalog refresh must retain the last-good payload")
            .0,
        "last-good"
    );
}

#[test]
fn an_old_lease_cannot_evict_a_re_registered_payload() {
    let manager = ResourceManager::new();
    let original = record("lease-model", "res://models/lease.glb");
    manager
        .commit(ready_batch(original.clone(), "old"))
        .expect("old payload");
    let handle = ResourceHandle::<ModelMarker>::new(original.id);
    let old_lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("old lease");

    manager
        .commit(ResourceMutationBatch::new().remove(original.primary_locator.clone()))
        .expect("remove old generation");
    manager
        .commit(ready_batch(original.clone(), "new"))
        .expect("new generation");
    drop(old_lease);

    assert_eq!(manager.ref_count(original.id), Some(0));
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Loaded)
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(handle)
            .expect("new payload survives stale lease release")
            .0,
        "new"
    );
}
