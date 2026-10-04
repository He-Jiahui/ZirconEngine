use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Duration;

use crate::{
    ModelMarker, ResourceEventKind, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
    ResourceReadinessState, ResourceRecord, RuntimeResourceState,
};

use super::{ResourceAuthority, ResourceManager};

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
fn resource_manager_accessors_recover_poisoned_state_locks() {
    let manager = ResourceManager::new();

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        manager.poison_event_stream_for_test();
    }));
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.lock_authority_write();
        panic!("poison resource authority");
    }));

    let events = manager.subscribe();
    let record = record("res://models/poisoned.obj", ResourceKind::Model);
    let id = record.id;
    let handle = manager
        .register_ready(
            record,
            TestPayload {
                name: "poisoned-ready",
            },
        )
        .expect("register ready payload")
        .typed::<ModelMarker>()
        .expect("typed model handle");

    let added = events
        .recv_timeout(Duration::from_secs(1))
        .expect("added event after poisoned subscriber lock");
    assert_eq!(added.kind, ResourceEventKind::Added);
    assert_eq!(added.id, id);
    assert_eq!(
        manager.registry().get(id).expect("record exists").revision,
        1
    );
    assert_eq!(
        manager
            .get::<ModelMarker, TestPayload>(ResourceHandle::new(id))
            .expect("payload remains accessible")
            .name,
        "poisoned-ready"
    );
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
    );

    let lease = manager
        .acquire::<ModelMarker, TestPayload>(handle)
        .expect("resource lease after poisoned runtime lock");
    assert_eq!(lease.name, "poisoned-ready");
    assert_eq!(manager.ref_count(id), Some(1));
    drop(lease);
    assert_eq!(manager.ref_count(id), Some(0));
    assert_eq!(
        manager.runtime_state(id),
        Some(RuntimeResourceState::Unloaded)
    );
}

#[test]
fn single_readiness_refresh_reuses_the_generation_when_source_is_unchanged() {
    let mut authority = ResourceAuthority::default();
    let record = record("res://models/readiness.glb", ResourceKind::Model);
    let id = record.id;
    assert!(authority.registry.insert_unchecked(record).is_none());

    authority.refresh_readiness_many([id]);
    let published = authority.readiness.generation();
    assert_eq!(published.diagnostics().publication_count, 1);
    assert!(published.contains_kind(id, ResourceKind::Model));

    authority.refresh_readiness_many([id]);
    assert!(Arc::ptr_eq(&published, &authority.readiness.generation()));
}

#[test]
fn empty_readiness_refresh_preserves_generation_identity() {
    let mut authority = ResourceAuthority::default();
    let published = authority.readiness.generation();

    authority.refresh_readiness_many(std::iter::empty::<ResourceId>());

    assert!(Arc::ptr_eq(&published, &authority.readiness.generation()));
    assert_eq!(published.diagnostics().publication_count, 0);
}

#[test]
fn duplicate_pair_readiness_refresh_publishes_one_row() {
    let mut authority = ResourceAuthority::default();
    let record = record("res://models/duplicate-readiness.glb", ResourceKind::Model);
    let id = record.id;
    assert!(authority.registry.insert_unchecked(record).is_none());

    authority.refresh_readiness_many([id, id]);

    let published = authority.readiness.generation();
    assert_eq!(published.diagnostics().publication_count, 1);
    assert_eq!(published.diagnostics().row_count, 1);
    assert_eq!(published.diagnostics().changed_row_count, 1);
    assert!(published.contains_kind(id, ResourceKind::Model));
}

#[test]
fn multi_id_readiness_refresh_keeps_peeked_ids_and_deduplicates() {
    let mut authority = ResourceAuthority::default();
    let model = record("res://models/multi-readiness.glb", ResourceKind::Model);
    let texture = record("res://textures/multi-readiness.png", ResourceKind::Texture);
    let shader = record("res://shaders/multi-readiness.wgsl", ResourceKind::Shader);
    let ids = [model.id, texture.id, shader.id];
    assert!(authority.registry.insert_unchecked(model).is_none());
    assert!(authority.registry.insert_unchecked(texture).is_none());
    assert!(authority.registry.insert_unchecked(shader).is_none());

    authority.refresh_readiness_many([ids[2], ids[0], ids[1], ids[2]]);

    let published = authority.readiness.generation();
    assert_eq!(published.diagnostics().publication_count, 1);
    assert_eq!(published.diagnostics().row_count, 3);
    assert!(published.contains_kind(ids[0], ResourceKind::Model));
    assert!(published.contains_kind(ids[1], ResourceKind::Texture));
    assert!(published.contains_kind(ids[2], ResourceKind::Shader));
    assert_eq!(published.diagnostics().changed_row_count, 3);

    authority.refresh_readiness_many([ids[0], ids[1], ids[2], ids[0]]);
    assert!(Arc::ptr_eq(&published, &authority.readiness.generation()));
}

#[test]
fn reversed_pair_refresh_prunes_lost_readiness_and_keeps_dangling_dependency_failed() {
    let mut authority = ResourceAuthority::default();
    let lost = record("res://textures/readiness-lost.png", ResourceKind::Texture);
    let lost_id = lost.id;
    let survivor = record("res://models/readiness-survivor.glb", ResourceKind::Model)
        .with_dependency_ids(vec![lost_id]);
    let survivor_id = survivor.id;
    assert!(authority.registry.insert_unchecked(lost).is_none());
    assert!(authority.registry.insert_unchecked(survivor).is_none());

    let (lower, higher) = if survivor_id < lost_id {
        (survivor_id, lost_id)
    } else {
        (lost_id, survivor_id)
    };
    authority.refresh_readiness_many([higher, lower]);

    let initial = authority.readiness.generation();
    assert_eq!(initial.diagnostics().row_count, 2);
    assert!(initial.contains_kind(lost_id, ResourceKind::Texture));
    assert!(initial.contains_kind(survivor_id, ResourceKind::Model));

    let unchanged = authority.readiness.generation();
    authority.refresh_readiness_many([survivor_id, survivor_id]);
    assert!(Arc::ptr_eq(&unchanged, &authority.readiness.generation()));

    assert!(authority.registry.remove_by_id(lost_id).is_some());
    authority.refresh_readiness_many([higher, lower]);

    let after_removal = authority.readiness.generation();
    assert!(after_removal.row_identity(lost_id).is_none());
    assert_eq!(after_removal.diagnostics().row_count, 1);
    let survivor = after_removal
        .row_identity(survivor_id)
        .expect("surviving readiness row");
    assert_eq!(
        survivor.row().direct_dependency_state,
        ResourceReadinessState::Failed
    );
    assert_eq!(
        survivor.row().recursive_dependency_state,
        ResourceReadinessState::Failed
    );
}

#[test]
fn single_readiness_refresh_does_not_buffer_and_sort_ids() {
    let source = include_str!("../resource_manager.rs");
    let refresh = source
        .split("pub(super) fn refresh_readiness_many")
        .nth(1)
        .and_then(|source| source.split("fn readiness_source_update").next())
        .expect("readiness refresh implementation");

    assert!(refresh.contains("let Some(second) = ids.next() else {"));
    assert!(refresh.contains("self.refresh_readiness_one(first);"));
    assert!(!refresh.contains("let mut ids = ids.into_iter().collect::<Vec<_>>();"));
}
