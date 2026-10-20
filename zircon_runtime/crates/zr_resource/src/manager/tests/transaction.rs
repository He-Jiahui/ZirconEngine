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
    assert!(manager
        .get::<ModelMarker, TestPayload>(ResourceHandle::new(original.id))
        .is_none());

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

// Complete source publication is the public watch boundary; no test-only authority view is used.
#[test]
fn m6_source_publication_preserves_last_good_and_reserves_exact_versions() {
    use crate::{
        MeshMarker, ResourceSourceMember, ResourceSourceOutcome, ResourceSourcePublication,
    };
    let manager = ResourceManager::new();
    let root = record("m6-source", "res://models/m6-source.glb")
        .with_state(ResourceState::Ready)
        .with_source_hash("a");
    let mut mesh = record("m6-source-mesh", "res://models/m6-source.glb#Mesh0")
        .with_state(ResourceState::Ready)
        .with_source_hash("a");
    mesh.kind = ResourceKind::Mesh;
    let mut cold = record("m6-source-cold", "res://models/m6-source.glb#Mesh1")
        .with_state(ResourceState::Ready)
        .with_source_hash("a");
    cold.kind = ResourceKind::Mesh;
    let members = vec![
        ResourceSourceMember {
            record: root.clone(),
            payload: Some(Arc::new(TestPayload("a-root"))),
        },
        ResourceSourceMember {
            record: mesh.clone(),
            payload: Some(Arc::new(TestPayload("a-mesh"))),
        },
    ];
    let initial = ResourceSourcePublication::capture(
        root.id,
        Vec::new(),
        members,
        ResourceSourceOutcome::CompleteSuccess,
        manager.projection_snapshot().active(),
    );
    manager
        .commit(ResourceMutationBatch::new().publish_source(initial))
        .unwrap();
    assert!(
        manager.get_untyped(root.id).is_none(),
        "owned importer output keeps cold members lazy"
    );
    let revision = manager.registry().get(root.id).unwrap().revision;
    manager
        .store_payload(root.id, revision, TestPayload("a-root"))
        .unwrap();
    let revision = manager.registry().get(mesh.id).unwrap().revision;
    manager
        .store_payload(mesh.id, revision, TestPayload("a-mesh"))
        .unwrap();
    let root_handle = ResourceHandle::<ModelMarker>::new(root.id);
    let mesh_handle = ResourceHandle::<MeshMarker>::new(mesh.id);
    let old_root = manager
        .acquire::<ModelMarker, TestPayload>(root_handle)
        .unwrap();
    let old_mesh = manager
        .acquire::<MeshMarker, TestPayload>(mesh_handle)
        .unwrap();
    let captured = manager.projection_snapshot();
    let root_key = captured.active().key(root.id).unwrap();
    let mesh_key = captured.active().key(mesh.id).unwrap();
    let invalid_success = ResourceSourcePublication::capture(
        root.id,
        vec![root.id, mesh.id],
        vec![ResourceSourceMember {
            record: root.clone().with_state(ResourceState::Error),
            payload: None,
        }],
        ResourceSourceOutcome::CompleteSuccess,
        captured.active(),
    );
    assert!(matches!(
        manager.commit(ResourceMutationBatch::new().publish_source(invalid_success)),
        Err(ResourceRegistryError::InvalidSourcePublication { .. })
    ));
    assert_eq!(
        manager.projection_snapshot().active().key(root.id).as_ref(),
        Some(&root_key)
    );

    let mut failed = root.clone().with_state(ResourceState::Error);
    failed.diagnostics = vec![ResourceDiagnostic::error("candidate import failed")];
    for _ in 0..3 {
        let publication = ResourceSourcePublication::capture(
            root.id,
            vec![root.id, mesh.id],
            vec![ResourceSourceMember {
                record: failed.clone(),
                payload: None,
            }],
            ResourceSourceOutcome::Failed,
            manager.projection_snapshot().active(),
        );
        manager
            .commit(ResourceMutationBatch::new().publish_source(publication))
            .unwrap();
        assert_eq!(
            manager.registry().get(root.id).unwrap().state,
            ResourceState::Error
        );
        assert!(
            manager.registry().get(mesh.id).is_none(),
            "candidate manifest still records failure"
        );
        let projection = manager.projection_snapshot();
        assert_eq!(projection.active().key(root.id).as_ref(), Some(&root_key));
        assert_eq!(projection.active().key(mesh.id).as_ref(), Some(&mesh_key));
        let fallback = manager
            .acquire::<MeshMarker, TestPayload>(mesh_handle)
            .unwrap();
        assert_eq!(fallback.resource().0, "a-mesh");
        drop(fallback);
        assert_eq!(
            manager.ref_count(mesh.id),
            Some(1),
            "metadata never increments leases"
        );
        // Both full-reconcile and clear-removed use this same protocol operation.
        manager
            .commit(
                ResourceMutationBatch::new()
                    .remove(mesh.primary_locator.clone())
                    .reconcile_sources([root.id]),
            )
            .unwrap();
        assert!(
            manager.get_untyped(mesh.id).is_some(),
            "failed active-only orphan survives candidate GC"
        );
    }
    assert_eq!(old_root.resource().0, "a-root");
    assert!(
        Arc::ptr_eq(captured.active(), manager.projection_snapshot().active()),
        "repeated failed publication and candidate GC keep the exact active generation"
    );
    assert!(manager
        .authority
        .read()
        .unwrap()
        .runtime
        .values()
        .all(|slot| slot.prepared.is_none()));

    // Last release in a prepared Ready batch is deferred, and abandon rechecks the real count.
    let unrelated = record("m6-reserved", "res://models/m6-reserved.glb");
    manager
        .commit(ready_batch(unrelated.clone(), "reserved"))
        .unwrap();
    let lease = manager
        .acquire::<ModelMarker, TestPayload>(ResourceHandle::new(unrelated.id))
        .unwrap();
    let unchanged = manager.registry().get(unrelated.id).unwrap().clone();
    let prepared = manager
        .prepare_commit(ResourceMutationBatch::new().upsert_lazy(unchanged))
        .unwrap();
    drop(lease); // Same-thread release must not try to acquire the serial lock.
    assert!(manager.get_untyped(unrelated.id).is_some());
    drop(prepared);
    assert!(manager.get_untyped(unrelated.id).is_none());
    assert!(manager
        .projection_snapshot()
        .active()
        .key(unrelated.id)
        .is_none());

    // Expected None is stable under the same serial writer gate and rejects an intervening install.
    manager
        .commit(ResourceMutationBatch::new().upsert_lazy(cold.clone()))
        .unwrap();
    let none_expected = ResourceSourcePublication::capture(
        cold.id,
        Vec::new(),
        vec![ResourceSourceMember {
            record: cold.clone(),
            payload: None,
        }],
        ResourceSourceOutcome::CompleteSuccess,
        manager.projection_snapshot().active(),
    );
    let revision = manager.registry().get(cold.id).unwrap().revision;
    manager
        .store_payload(cold.id, revision, TestPayload("intervening"))
        .unwrap();
    assert!(matches!(
        manager.commit(ResourceMutationBatch::new().publish_source(none_expected)),
        Err(ResourceRegistryError::ActiveVersionConflict { .. })
    ));
    manager
        .commit(ResourceMutationBatch::new().remove(cold.primary_locator.clone()))
        .unwrap();

    // Success replaces only residents, keeps new members cold and removes omitted old subassets.
    let success = ResourceSourcePublication::capture(
        root.id,
        vec![root.id, mesh.id],
        vec![
            ResourceSourceMember {
                record: root.clone().with_source_hash("b"),
                payload: Some(Arc::new(TestPayload("b-root"))),
            },
            ResourceSourceMember {
                record: cold.clone().with_source_hash("b"),
                payload: Some(Arc::new(TestPayload("b-cold"))),
            },
        ],
        ResourceSourceOutcome::CompleteSuccess,
        manager.projection_snapshot().active(),
    );
    manager
        .commit(ResourceMutationBatch::new().publish_source(success))
        .unwrap();
    assert_ne!(
        manager.projection_snapshot().active().key(root.id).as_ref(),
        Some(&root_key)
    );
    assert!(manager.get_untyped(mesh.id).is_none());
    assert!(manager.get_untyped(cold.id).is_none());
    assert_eq!(
        manager.registry().get(cold.id).unwrap().state,
        ResourceState::Ready
    );
    assert_eq!(manager.ref_count(root.id), Some(0));
    drop(old_root);
    drop(old_mesh);
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(root_handle)
            .unwrap()
            .0,
        "b-root"
    );
    let b = manager
        .acquire::<ModelMarker, TestPayload>(root_handle)
        .unwrap();
    assert_eq!(b.resource().0, "b-root");
    drop(b);
    assert!(
        manager.get_untyped(root.id).is_none(),
        "B's first real acquire/last drop keeps Ready semantics"
    );

    let removed = ResourceSourcePublication::capture(
        root.id,
        Vec::new(),
        Vec::new(),
        ResourceSourceOutcome::Removed,
        manager.projection_snapshot().active(),
    );
    manager
        .commit(
            ResourceMutationBatch::new()
                .publish_source(removed)
                .reconcile_sources([]),
        )
        .unwrap();
    assert!(manager
        .projection_snapshot()
        .active()
        .source_members(root.id)
        .is_empty());
    assert!(manager.registry().get(root.id).is_none());
    assert!(manager.registry().get(cold.id).is_none());

    // A failed preflight owns candidate bytes until after both locks have been released.
    struct DropProbe {
        manager: std::sync::Weak<ResourceManager>,
        dropped: Arc<AtomicBool>,
    }
    impl std::fmt::Debug for DropProbe {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("DropProbe")
        }
    }
    impl Drop for DropProbe {
        fn drop(&mut self) {
            let manager = self.manager.upgrade().unwrap();
            assert!(!manager.commit_gate_is_locked_for_test());
            assert!(
                manager.authority.try_write().is_ok(),
                "candidate final Drop must not hold authority"
            );
            self.dropped.store(true, Ordering::Release);
        }
    }
    let manager = Arc::new(ResourceManager::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let installed = record("m6-drop-root", "res://models/m6-drop-root.glb");
    manager
        .commit(ready_batch(installed.clone(), "original"))
        .unwrap();
    let mut wrong_kind = installed;
    wrong_kind.kind = ResourceKind::Mesh;
    let result = manager.commit(
        ResourceMutationBatch::new()
            .upsert_ready(
                record("m6-drop-probe", "res://models/m6-drop-probe.glb"),
                DropProbe {
                    manager: Arc::downgrade(&manager),
                    dropped: Arc::clone(&dropped),
                },
            )
            .upsert_lazy(wrong_kind),
    );
    assert!(matches!(
        result,
        Err(ResourceRegistryError::KindConflict { .. })
    ));
    assert!(dropped.load(Ordering::Acquire));
}

#[test]
fn declared_payload_mismatch_rejects_entire_batch_before_publication() {
    let manager = ResourceManager::new();
    let candidate =
        record("declared-model", "res://models/declared.glb").with_state(ResourceState::Ready);
    manager
        .commit(ResourceMutationBatch::new().upsert_typed_lazy::<TestPayload>(candidate.clone()))
        .unwrap();
    let revision = manager.registry().get(candidate.id).unwrap().revision;
    let untouched = record("must-not-publish", "res://models/unpublished.glb");
    let events = manager.subscribe();
    let management = manager.management_generation();
    let readiness = manager.readiness_generation();
    let error = manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(untouched.clone())
                .store_payload(candidate.id, revision, 17_u32),
        )
        .unwrap_err();
    assert_eq!(
        error,
        ResourceRegistryError::PayloadTypeConflict {
            id: candidate.id.to_string()
        }
    );
    assert!(manager.registry().get(untouched.id).is_none());
    assert!(manager
        .registry()
        .contains_payload::<TestPayload>(candidate.id));
    assert!(manager
        .get::<ModelMarker, TestPayload>(ResourceHandle::new(candidate.id))
        .is_none());
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert!(Arc::ptr_eq(&readiness, &manager.readiness_generation()));
    assert_eq!(events.try_recv(), Err(ResourceEventTryRecvError::Empty));
    let conflict = manager
        .commit(
            ResourceMutationBatch::new()
                .remove(candidate.primary_locator.clone())
                .upsert_typed_lazy::<u32>(candidate.clone()),
        )
        .unwrap_err();
    assert_eq!(
        conflict,
        ResourceRegistryError::PayloadTypeConflict {
            id: candidate.id.to_string()
        }
    );
    assert!(Arc::ptr_eq(&management, &manager.management_generation()));
    assert_eq!(events.try_recv(), Err(ResourceEventTryRecvError::Empty));
    manager
        .commit(ResourceMutationBatch::new().store_payload(
            candidate.id,
            revision,
            TestPayload("recovered"),
        ))
        .unwrap();
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(candidate.id))
            .unwrap()
            .0,
        "recovered"
    );
}
