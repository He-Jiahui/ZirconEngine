use std::sync::{mpsc, Arc, Barrier};
use std::thread;

use crate::core::resource::{
    ModelMarker, ResourceEventKind, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
    ResourceManager, ResourceMutationBatch, ResourceRecord,
};

#[derive(Debug, PartialEq, Eq)]
struct TestPayload(&'static str);

#[test]
fn prepared_commit_stays_private_and_holds_the_shared_publication_gate() {
    let manager = Arc::new(ResourceManager::new());
    let first = ResourceRecord::new(
        ResourceId::from_stable_label("prepared-model"),
        ResourceKind::Model,
        ResourceLocator::parse("res://models/prepared.glb").unwrap(),
    );
    let second = ResourceRecord::new(
        ResourceId::from_stable_label("serialized-model"),
        ResourceKind::Model,
        ResourceLocator::parse("res://models/serialized.glb").unwrap(),
    );
    let events = manager.subscribe();
    let prepared = manager
        .prepare_commit(
            ResourceMutationBatch::new().upsert_ready(first.clone(), TestPayload("prepared")),
        )
        .expect("resource preflight should reserve the commit gate");
    assert!(manager.commit_gate_is_locked_for_test());
    assert!(manager.registry().get(first.id).is_none());
    assert!(events.try_recv().is_err());

    let started = Arc::new(Barrier::new(2));
    let worker_started = Arc::clone(&started);
    let worker_manager = Arc::clone(&manager);
    let (completed_tx, completed_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        worker_started.wait();
        let result = worker_manager
            .commit(ResourceMutationBatch::new().upsert_ready(second, TestPayload("serialized")));
        completed_tx.send(result).unwrap();
    });

    started.wait();
    let receipt = prepared.commit();
    let committed = receipt
        .record(first.id)
        .expect("prepared commit publishes its record");
    assert_eq!(committed.id, first.id);
    assert_eq!(committed.primary_locator, first.primary_locator);
    assert_eq!(committed.state, crate::core::resource::ResourceState::Ready);
    assert_eq!(committed.revision, 1);
    completed_rx
        .recv()
        .expect("worker reports completion")
        .expect("commit succeeds after publication gate release");
    worker.join().unwrap();

    let first_event = events.recv().expect("prepared commit event");
    let second_event = events.recv().expect("serialized follower event");
    assert_eq!(first_event.kind, ResourceEventKind::Added);
    assert_eq!(first_event.id, first.id);
    assert_eq!(second_event.kind, ResourceEventKind::Added);
    assert_eq!(
        second_event.id,
        ResourceId::from_stable_label("serialized-model")
    );
}

#[test]
fn dropping_a_prepared_commit_releases_the_gate_without_publishing_state() {
    let manager = ResourceManager::new();
    let abandoned = ResourceRecord::new(
        ResourceId::from_stable_label("abandoned-prepared-model"),
        ResourceKind::Model,
        ResourceLocator::parse("res://models/abandoned-prepared.glb").unwrap(),
    );
    let committed = ResourceRecord::new(
        ResourceId::from_stable_label("commit-after-abandon"),
        ResourceKind::Model,
        ResourceLocator::parse("res://models/commit-after-abandon.glb").unwrap(),
    );
    let events = manager.subscribe();
    let prepared = manager
        .prepare_commit(
            ResourceMutationBatch::new().upsert_ready(abandoned.clone(), TestPayload("abandoned")),
        )
        .expect("resource preflight should succeed");

    drop(prepared);

    assert!(!manager.commit_gate_is_locked_for_test());
    assert!(manager.registry().get(abandoned.id).is_none());
    assert!(events.try_recv().is_err());
    manager
        .commit(
            ResourceMutationBatch::new().upsert_ready(committed.clone(), TestPayload("committed")),
        )
        .expect("a failed outer transaction must not strand the commit gate");
    assert_eq!(manager.registry().get(committed.id), Some(&committed));
    assert_eq!(
        events.recv().expect("successful follower event").id,
        committed.id
    );
}

#[test]
fn stale_release_after_concurrent_remove_and_re_register_is_generation_scoped() {
    let manager = Arc::new(ResourceManager::new());
    let locator = ResourceLocator::parse("res://models/concurrent.glb").unwrap();
    let record = ResourceRecord::new(
        ResourceId::from_stable_label("concurrent-model"),
        ResourceKind::Model,
        locator.clone(),
    );
    manager
        .commit(ResourceMutationBatch::new().upsert_ready(record.clone(), TestPayload("old")))
        .unwrap();
    let handle = ResourceHandle::<ModelMarker>::new(record.id);
    let acquired = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker_manager = Arc::clone(&manager);
    let worker_acquired = Arc::clone(&acquired);
    let worker_release = Arc::clone(&release);
    let worker = thread::spawn(move || {
        let lease = worker_manager
            .acquire::<ModelMarker, TestPayload>(handle)
            .expect("old payload lease");
        worker_acquired.wait();
        worker_release.wait();
        drop(lease);
    });

    acquired.wait();
    manager
        .commit(ResourceMutationBatch::new().remove(locator))
        .unwrap();
    manager
        .commit(ResourceMutationBatch::new().upsert_ready(record, TestPayload("new")))
        .unwrap();
    release.wait();
    worker.join().unwrap();

    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(handle)
            .expect("new payload remains resident")
            .0,
        "new"
    );
}
