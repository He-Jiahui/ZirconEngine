use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use crate::{
    ModelMarker, ResourceDiagnostic, ResourceEventKind, ResourceEventTryRecvError, ResourceHandle,
    ResourceId, ResourceKind, ResourceLocator, ResourceManagementGeneration,
    ResourceManagementQuery, ResourceManager, ResourceMutationBatch, ResourceReadinessGeneration,
    ResourceRecord, ResourceRegistryError, ResourceState, RuntimeResourceState,
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
fn event_sequence_exhaustion_rejects_a_commit_before_resource_mutation() {
    let manager = ResourceManager::new();
    manager.set_event_next_sequence_for_test(None);
    let candidate = record("terminal-rejected", "res://models/terminal-rejected.glb");
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();

    let error = manager
        .commit(ResourceMutationBatch::new().upsert_lazy(candidate.clone()))
        .expect_err("an event-producing commit must fail after sequence exhaustion");

    assert_eq!(
        error,
        ResourceRegistryError::EventSequenceExhausted {
            requested_event_count: 1,
        }
    );
    assert!(manager.registry().get(candidate.id).is_none());
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
    let diagnostics = manager.event_stream_diagnostics();
    assert!(diagnostics.sequence_exhausted);
    assert_eq!(diagnostics.rejected_publish_count, 1);
}

#[test]
fn event_free_commit_remains_available_after_event_sequence_exhaustion() {
    let manager = ResourceManager::new();
    manager.set_event_next_sequence_for_test(None);
    let transient = record("terminal-noop", "res://models/terminal-noop.glb");

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(transient.clone())
                .remove(transient.primary_locator.clone()),
        )
        .expect("a commit without an observable event remains valid");

    assert_eq!(receipt.published_event_count(), 0);
    assert!(manager.registry().get(transient.id).is_none());
    assert_eq!(manager.event_stream_diagnostics().rejected_publish_count, 0);
}

#[test]
fn dropping_a_prepared_commit_does_not_consume_the_final_event_sequence() {
    let manager = ResourceManager::new();
    manager.set_event_next_sequence_for_test(Some(u64::MAX));
    let receiver = manager.subscribe();
    let abandoned = record("abandoned-terminal", "res://models/abandoned-terminal.glb");
    let committed = record("committed-terminal", "res://models/committed-terminal.glb");

    let prepared = manager
        .prepare_commit(ResourceMutationBatch::new().upsert_lazy(abandoned.clone()))
        .expect("the final sequence can be reserved without mutation");
    drop(prepared);
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(committed.clone()))
        .expect("dropping the prepared commit leaves the final sequence available");

    assert!(manager.registry().get(abandoned.id).is_none());
    assert!(manager.registry().get(committed.id).is_some());
    assert_eq!(receiver.try_recv().unwrap().id, committed.id);
    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::SequenceExhausted)
    );
}

#[test]
fn a_commit_needing_more_than_the_remaining_event_range_is_atomic() {
    let manager = ResourceManager::new();
    manager.set_event_next_sequence_for_test(Some(u64::MAX));
    let first = record(
        "terminal-range-first",
        "res://models/terminal-range-first.glb",
    );
    let second = record(
        "terminal-range-second",
        "res://models/terminal-range-second.glb",
    );

    let error = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(first.clone())
                .upsert_lazy(second.clone()),
        )
        .expect_err("the entire publication range must be admitted atomically");

    assert_eq!(
        error,
        ResourceRegistryError::EventSequenceExhausted {
            requested_event_count: 2,
        }
    );
    assert!(manager.registry().get(first.id).is_none());
    assert!(manager.registry().get(second.id).is_none());
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
    assert!(
        manager
            .registry()
            .get(ResourceId::from_stable_label("model-c"))
            .is_none()
    );
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
fn remove_then_ready_readd_preserves_revision_cas_lineage() {
    let manager = ResourceManager::new();
    let original = record(
        "revision-lineage-model",
        "res://models/revision-lineage.glb",
    )
    .with_source_hash("v1");
    manager
        .commit(ready_batch(original.clone(), "v1"))
        .expect("initial ready record");
    let stale_revision = manager
        .registry()
        .get(original.id)
        .expect("initial record")
        .revision;
    let events = manager.subscribe();

    let replacement = original.clone().with_source_hash("v2");
    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .remove(original.primary_locator.clone())
                .upsert_ready(replacement, TestPayload("v2")),
        )
        .expect("atomic remove and ready re-add");

    let published_revision = receipt
        .record(original.id)
        .expect("replacement record")
        .revision;
    assert_eq!(published_revision, stale_revision + 1);
    let event = events.recv().expect("replacement event");
    assert_eq!(event.kind, ResourceEventKind::Updated);
    assert_eq!(event.revision, published_revision);

    let error = manager
        .commit(ResourceMutationBatch::new().store_payload(
            original.id,
            stale_revision,
            TestPayload("stale-v1"),
        ))
        .expect_err("the pre-batch revision must not authorize a replacement payload");
    assert_eq!(
        error,
        ResourceRegistryError::RevisionConflict {
            id: original.id.to_string(),
            expected_revision: stale_revision,
            actual_revision: Some(published_revision),
        }
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(original.id))
            .expect("replacement payload remains installed")
            .0,
        "v2"
    );
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

    let published = manager.projection_snapshot();
    assert_eq!(
        receipt.projection_snapshot().management_identity(),
        published.management_identity()
    );
    assert_eq!(
        receipt.projection_snapshot().readiness_identity(),
        published.readiness_identity()
    );
    assert_eq!(receipt.published_event_count(), 3);
    assert_eq!(
        manager
            .management_generation()
            .diagnostics()
            .publication_count,
        1
    );
    assert_eq!(
        manager
            .readiness_generation()
            .diagnostics()
            .publication_count,
        1
    );
    assert_eq!(manager.management_generation().summary().total_count(), 3);
}

#[test]
fn projection_snapshot_page_and_receipt_retain_exact_generation_identities() {
    let manager = ResourceManager::new();
    let before = manager.projection_snapshot();
    let model = record("identity-model", "res://models/identity.glb");

    let receipt = manager
        .commit(ready_batch(model, "identity"))
        .expect("identity publication");
    let after = manager.projection_snapshot();
    let page = after
        .management()
        .page(ResourceManagementQuery::default(), 0, usize::MAX);

    assert_ne!(before.management_identity(), after.management_identity());
    assert_ne!(before.readiness_identity(), after.readiness_identity());
    assert_eq!(
        receipt.projection_snapshot().management_identity(),
        after.management_identity()
    );
    assert_eq!(
        receipt.projection_snapshot().readiness_identity(),
        after.readiness_identity()
    );
    assert_eq!(page.generation, after.management_identity());
}

#[test]
fn equal_diagnostic_sequences_do_not_alias_distinct_generation_objects() {
    let first_management = Arc::new(ResourceManagementGeneration::default());
    let second_management = Arc::new(ResourceManagementGeneration::default());
    let first_readiness = Arc::new(ResourceReadinessGeneration::default());
    let second_readiness = Arc::new(ResourceReadinessGeneration::default());

    assert_eq!(
        first_management.diagnostics().publication_count,
        second_management.diagnostics().publication_count
    );
    assert_eq!(
        first_readiness.diagnostics().publication_count,
        second_readiness.diagnostics().publication_count
    );
    assert_ne!(first_management.identity(), second_management.identity());
    assert_ne!(first_readiness.identity(), second_readiness.identity());
}

#[test]
fn unrelated_publication_reuses_exact_management_and_readiness_row_identities() {
    let manager = ResourceManager::new();
    let stable = record("stable-identity-model", "res://models/stable-identity.glb");
    let changing = record(
        "changing-identity-model",
        "res://models/changing-identity.glb",
    )
    .with_source_hash("v1");
    let changing_id = changing.id;
    manager
        .commit(
            ready_batch(stable.clone(), "stable")
                .upsert_ready(changing.clone(), TestPayload("changing-v1")),
        )
        .expect("initial identity publication");
    let before = manager.projection_snapshot();
    let stable_management = before
        .management()
        .row_identity_by_id(stable.id)
        .expect("stable management row");
    let stable_readiness = before
        .readiness()
        .row_identity(stable.id)
        .expect("stable readiness row");
    let changing_management = before
        .management()
        .row_identity_by_id(changing_id)
        .expect("changing management row");
    let changing_readiness = before
        .readiness()
        .row_identity(changing_id)
        .expect("changing readiness row");

    manager
        .commit(ready_batch(changing.with_source_hash("v2"), "changing-v2"))
        .expect("unrelated identity publication");
    let after = manager.projection_snapshot();

    assert_eq!(
        stable_management,
        after
            .management()
            .row_identity_by_id(stable.id)
            .expect("reused management row")
    );
    assert_eq!(
        stable_readiness,
        after
            .readiness()
            .row_identity(stable.id)
            .expect("reused readiness row")
    );
    assert_ne!(
        changing_management,
        after
            .management()
            .row_identity_by_id(changing_id)
            .expect("changed management row")
    );
    assert_ne!(
        changing_readiness,
        after
            .readiness()
            .row_identity(changing_id)
            .expect("changed readiness row")
    );
}

#[test]
fn projection_snapshot_never_combines_management_and_readiness_from_different_commits() {
    let manager = Arc::new(ResourceManager::new());
    let finished = Arc::new(AtomicBool::new(false));
    let writer_manager = Arc::clone(&manager);
    let writer_finished = Arc::clone(&finished);
    let writer = thread::spawn(move || {
        for index in 0..256 {
            let model = record(
                &format!("paired-snapshot-{index}"),
                &format!("res://models/paired-snapshot-{index}.glb"),
            );
            writer_manager
                .commit(ResourceMutationBatch::new().upsert_lazy(model))
                .expect("paired snapshot publication");
        }
        writer_finished.store(true, Ordering::Release);
    });

    while !finished.load(Ordering::Acquire) {
        let snapshot = manager.projection_snapshot();
        assert_eq!(
            snapshot.management().summary().total_count(),
            snapshot.readiness().diagnostics().row_count
        );
    }
    writer.join().expect("paired snapshot writer");
    let snapshot = manager.projection_snapshot();
    assert_eq!(snapshot.management().summary().total_count(), 256);
    assert_eq!(snapshot.readiness().diagnostics().row_count, 256);
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
fn importer_identity_change_advances_revision_and_invalidates_old_payload() {
    let manager = ResourceManager::new();
    let original = record("importer-model", "res://models/importer.glb")
        .with_source_hash("stable-source")
        .with_importer_id("importer-a");
    manager
        .commit(ready_batch(original.clone(), "importer-a-payload"))
        .expect("initial importer payload");
    let current = manager.registry().get(original.id).unwrap().clone();
    let stale_revision = current.revision;
    let changed = current.with_importer_id("importer-b");

    let receipt = manager
        .commit(ResourceMutationBatch::new().upsert_lazy(changed))
        .expect("importer identity update");

    let published_revision = receipt.record(original.id).unwrap().revision;
    assert_eq!(published_revision, stale_revision + 1);
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Unloaded)
    );
    assert!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(original.id))
            .is_none()
    );

    let error = manager
        .commit(ResourceMutationBatch::new().store_payload(
            original.id,
            stale_revision,
            TestPayload("stale-importer-payload"),
        ))
        .expect_err("the previous importer revision must not authorize payload publication");
    assert_eq!(
        error,
        ResourceRegistryError::RevisionConflict {
            id: original.id.to_string(),
            expected_revision: stale_revision,
            actual_revision: Some(published_revision),
        }
    );
}

#[test]
fn ready_revision_exhaustion_rejects_the_entire_batch() {
    let manager = ResourceManager::new();
    let mut saturated = record("saturated-model", "res://models/saturated.glb")
        .with_state(ResourceState::Ready)
        .with_source_hash("v1");
    saturated.revision = u64::MAX;
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(saturated.clone()))
        .expect("restore a persisted saturated revision");
    let events = manager.subscribe();
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();
    let changed = saturated.clone().with_source_hash("v2");

    let error = manager
        .commit(ResourceMutationBatch::new().upsert_lazy(changed))
        .expect_err("a saturated ready revision must fail closed");

    assert_eq!(
        error,
        ResourceRegistryError::RevisionExhausted {
            id: saturated.id.to_string(),
            current_revision: u64::MAX,
        }
    );
    assert_eq!(manager.registry().get(saturated.id), Some(&saturated));
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
    assert!(events.try_recv().is_err());
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

    let remove_readd_error = manager
        .commit(
            ResourceMutationBatch::new()
                .remove(failed.primary_locator.clone())
                .upsert_ready(failed.clone(), TestPayload("remove-readd")),
        )
        .expect_err("batch-local removal cannot bypass explicit reload recovery");
    assert!(matches!(
        remove_readd_error,
        ResourceRegistryError::InvalidStateTransition { .. }
    ));
    assert_eq!(
        manager.registry().get(failed.id).map(|record| record.state),
        Some(ResourceState::Error)
    );

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
fn explicit_reload_transition_survives_remove_and_readd() {
    let manager = ResourceManager::new();
    let failed = record(
        "remove-readd-recovery-model",
        "res://models/remove-readd-recovery.glb",
    )
    .with_state(ResourceState::Error)
    .with_source_hash("stable-source");
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(failed.clone()))
        .expect("failed record");
    assert_eq!(manager.registry().get(failed.id).unwrap().revision, 0);

    let receipt = manager
        .commit(
            ResourceMutationBatch::new()
                .start_reload(failed.id, Vec::new())
                .remove(failed.primary_locator.clone())
                .upsert_ready(failed.clone(), TestPayload("recovered")),
        )
        .expect("explicit reload transition authorizes the final ready record");

    let recovered = receipt.record(failed.id).expect("recovered record");
    assert_eq!(recovered.state, ResourceState::Ready);
    assert_eq!(recovered.revision, 1);
    assert_eq!(
        manager.runtime_state(failed.id),
        Some(RuntimeResourceState::Loaded)
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(failed.id))
            .expect("recovered payload")
            .0,
        "recovered"
    );
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

#[test]
fn an_old_lease_cannot_evict_a_directly_replaced_payload() {
    let manager = ResourceManager::new();
    let original = record(
        "lease-replacement-model",
        "res://models/lease-replacement.glb",
    );
    manager
        .commit(ready_batch(original.clone(), "old"))
        .expect("old payload");
    let handle = ResourceHandle::<ModelMarker>::new(original.id);
    let old_lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("old lease");

    manager
        .commit(ready_batch(original.clone(), "new"))
        .expect("replacement payload");
    drop(old_lease);

    assert_eq!(manager.ref_count(original.id), Some(0));
    assert_eq!(
        manager.runtime_state(original.id),
        Some(RuntimeResourceState::Loaded)
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(handle)
            .expect("replacement payload survives stale lease release")
            .0,
        "new"
    );
}

#[test]
fn concurrent_final_lease_releases_unload_the_current_payload() {
    let manager = ResourceManager::new();
    let resource = record(
        "concurrent-lease-model",
        "res://models/concurrent-lease.glb",
    );
    manager
        .commit(ready_batch(resource.clone(), "current"))
        .expect("current payload");
    let handle = ResourceHandle::<ModelMarker>::new(resource.id);
    let first = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("first lease");
    let second = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("second lease");
    let barrier = Arc::new(Barrier::new(3));

    let first_barrier = Arc::clone(&barrier);
    let first_drop = thread::spawn(move || {
        first_barrier.wait();
        drop(first);
    });
    let second_barrier = Arc::clone(&barrier);
    let second_drop = thread::spawn(move || {
        second_barrier.wait();
        drop(second);
    });
    barrier.wait();
    first_drop.join().expect("first lease drop");
    second_drop.join().expect("second lease drop");

    assert_eq!(manager.ref_count(resource.id), Some(0));
    assert_eq!(
        manager.runtime_state(resource.id),
        Some(RuntimeResourceState::Unloaded)
    );
    assert!(manager.get_untyped(resource.id).is_none());
}
