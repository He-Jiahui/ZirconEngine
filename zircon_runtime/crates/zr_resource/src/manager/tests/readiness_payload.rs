use std::any::TypeId;
use std::sync::Arc;

use super::ResourceManager;
use crate::{
    ModelMarker, ResourceId, ResourceKind, ResourceLocator, ResourceMutationBatch,
    ResourceReadinessGeneration, ResourceReadinessState, ResourceRecord, ResourceState,
    RuntimeResourceState,
};

use super::super::readiness_projection::ResourceReadinessProjection;

#[derive(Debug, PartialEq, Eq)]
struct TestPayload(&'static str);

fn record(label: &str) -> ResourceRecord {
    let locator = ResourceLocator::parse(&format!("res://readiness/{label}.asset"))
        .expect("valid readiness locator");
    ResourceRecord::new(
        ResourceId::from_locator(&locator),
        ResourceKind::Model,
        locator,
    )
}

#[test]
fn readiness_source_update_records_the_erased_payload_type() {
    let manager = ResourceManager::new();
    let record = record("concrete-type");
    let id = record.id;
    let handle = manager
        .register_ready(record, TestPayload("loaded"))
        .unwrap()
        .typed::<ModelMarker>()
        .unwrap();

    assert_eq!(manager.get::<_, TestPayload>(handle).unwrap().0, "loaded");
    let authority = manager.lock_authority_read();
    assert_eq!(
        authority.readiness_source_update(id).payload_type_id,
        Some(TypeId::of::<TestPayload>())
    );
}

#[test]
fn replacing_payload_type_publishes_typed_readiness_and_preserves_old_generation() {
    let manager = ResourceManager::new();
    let record = record("replacement-type");
    let id = record.id;
    manager
        .register_ready(record, TestPayload("original"))
        .unwrap();

    let previous = manager.readiness_generation();
    let previous_row = previous.row(id).unwrap();
    assert_eq!(previous_row.load_state, ResourceReadinessState::Loaded);
    assert_eq!(
        previous_row.typed_load_state::<TestPayload>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        previous_row.typed_load_state::<u64>(),
        ResourceReadinessState::NotLoaded
    );

    manager
        .store_payload(id, previous_row.record.revision, 17_u64)
        .unwrap();

    let current = manager.readiness_generation();
    assert!(!Arc::ptr_eq(&previous, &current));
    let current_row = current.row(id).unwrap();
    assert_eq!(
        current_row.typed_load_state::<u64>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        current_row.typed_load_state::<TestPayload>(),
        ResourceReadinessState::NotLoaded
    );
    assert_eq!(
        previous_row.typed_load_state::<TestPayload>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        previous_row.typed_load_state::<u64>(),
        ResourceReadinessState::NotLoaded
    );
}

#[test]
fn an_arc_payload_retains_its_own_concrete_type() {
    let manager = ResourceManager::new();
    let record = record("arc-value");
    let id = record.id;
    manager
        .register_ready(record, Arc::new(TestPayload("nested")))
        .unwrap();

    let generation = manager.readiness_generation();
    let row = generation.row(id).unwrap();
    assert_eq!(
        row.typed_load_state::<Arc<TestPayload>>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        row.typed_load_state::<TestPayload>(),
        ResourceReadinessState::NotLoaded
    );
}

struct ReadyFixture {
    manager: ResourceManager,
    buffered: ResourceReadinessProjection,
    // leaf -> parent -> root; peer and spare have no dependencies.
    ids: [ResourceId; 5],
}

impl ReadyFixture {
    fn new() -> Self {
        let leaf = record("transition-leaf");
        let parent = record("transition-parent").with_dependency_ids(vec![leaf.id]);
        let root = record("transition-root").with_dependency_ids(vec![parent.id]);
        let peer = record("transition-peer");
        let spare = record("transition-spare");
        let ids = [leaf.id, parent.id, root.id, peer.id, spare.id];
        let mut fixture = Self {
            manager: ResourceManager::new(),
            buffered: ResourceReadinessProjection::default(),
            ids,
        };
        for record in [leaf, parent, root, peer, spare] {
            let id = record.id;
            fixture
                .manager
                .register_ready(record, TestPayload("original"))
                .expect("real ready registration");
            fixture.assert_parity([id]);
        }
        fixture.assert_loaded();
        fixture
    }

    fn assert_parity(&mut self, ids: impl IntoIterator<Item = ResourceId>) {
        // Retain the previous materialized source-update path as a semantic reference.
        let mut ids = ids.into_iter().collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        let updates = {
            let authority = self.manager.lock_authority_read();
            ids.into_iter()
                .map(|id| authority.readiness_source_update(id))
                .collect::<Vec<_>>()
        };
        self.buffered.apply_updates(updates);
        let candidate = self.manager.readiness_generation();
        let buffered = self.buffered.generation();
        assert_eq!(candidate.diagnostics(), buffered.diagnostics());
        for id in self.ids {
            let actual = candidate.row(id);
            let expected = buffered.row(id);
            match (actual, expected) {
                (Some(actual), Some(expected)) => {
                    assert_eq!(actual.record, expected.record);
                    assert_eq!(actual.load_state, expected.load_state);
                    assert_eq!(
                        actual.direct_dependency_state,
                        expected.direct_dependency_state
                    );
                    assert_eq!(
                        actual.recursive_dependency_state,
                        expected.recursive_dependency_state
                    );
                    assert_eq!(actual.payload_type_id, expected.payload_type_id);
                    assert_eq!(actual.dependency_revision, expected.dependency_revision);
                    assert_eq!(
                        actual.dependency_fingerprint,
                        expected.dependency_fingerprint
                    );
                }
                (None, None) => {}
                _ => panic!("readiness row presence differs for {id:?}"),
            }
        }
    }

    fn assert_loaded(&self) {
        let generation = self.manager.readiness_generation();
        for id in self.ids {
            let row = generation.row(id).expect("registered readiness row");
            assert_eq!(row.record.state, ResourceState::Ready);
            assert_eq!(row.load_state, ResourceReadinessState::Loaded);
            assert_eq!(row.direct_dependency_state, ResourceReadinessState::Loaded);
            assert_eq!(
                row.recursive_dependency_state,
                ResourceReadinessState::Loaded
            );
            assert_eq!(
                self.manager.runtime_state(id),
                Some(RuntimeResourceState::Loaded)
            );
            assert!(self.manager.get_untyped(id).is_some());
        }
    }

    fn reload(&mut self, ids: &[ResourceId], fail: bool) {
        let before = self.manager.readiness_generation();
        let mut batch = ResourceMutationBatch::new();
        for &id in ids {
            batch = if fail {
                batch.fail_reload(id, Vec::new())
            } else {
                batch.start_reload(id, Vec::new())
            };
        }
        self.manager.commit(batch).expect("valid reload transition");
        self.assert_parity(ids.iter().copied());
        assert_eq!(
            self.manager
                .readiness_generation()
                .diagnostics()
                .publication_count,
            before.diagnostics().publication_count + 1
        );
    }

    fn retry_ready(&mut self, ids: &[ResourceId]) {
        self.reload(ids, false);
        let reloading = self.manager.readiness_generation();
        let mut batch = ResourceMutationBatch::new();
        for &id in ids {
            let record = self.manager.registry().get(id).unwrap().clone();
            batch = batch.upsert_ready(record, TestPayload("retry"));
        }
        self.manager.commit(batch).expect("publish retry payloads");
        self.assert_parity(ids.iter().copied());
        self.assert_loaded();
        assert_eq!(
            self.manager
                .readiness_generation()
                .diagnostics()
                .publication_count,
            reloading.diagnostics().publication_count + 1
        );
    }

    fn assert_unchanged_refresh(&mut self, ids: impl IntoIterator<Item = ResourceId>) {
        let ids = ids.into_iter().collect::<Vec<_>>();
        let published = self.manager.readiness_generation();
        let buffered = self.buffered.generation();
        self.manager
            .lock_authority_write()
            .refresh_readiness_many(ids.iter().copied());
        self.assert_parity(ids);
        assert!(Arc::ptr_eq(
            &published,
            &self.manager.readiness_generation()
        ));
        assert!(Arc::ptr_eq(&buffered, &self.buffered.generation()));
    }
}

fn assert_dependency_failure(generation: &ResourceReadinessGeneration, ids: [ResourceId; 5]) {
    let [leaf, parent, root, _, _] = ids;
    let leaf_row = generation.row(leaf).unwrap();
    assert_eq!(leaf_row.record.state, ResourceState::Error);
    assert_eq!(leaf_row.load_state, ResourceReadinessState::Failed);
    assert_eq!(
        leaf_row.typed_load_state::<TestPayload>(),
        ResourceReadinessState::Failed
    );
    let parent_row = generation.row(parent).unwrap();
    assert_eq!(parent_row.load_state, ResourceReadinessState::Loaded);
    assert_eq!(
        parent_row.direct_dependency_state,
        ResourceReadinessState::Failed
    );
    assert_eq!(
        parent_row.recursive_dependency_state,
        ResourceReadinessState::Failed
    );
    let root_row = generation.row(root).unwrap();
    assert_eq!(root_row.load_state, ResourceReadinessState::Loaded);
    assert_eq!(
        root_row.direct_dependency_state,
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        root_row.recursive_dependency_state,
        ResourceReadinessState::Failed
    );
}

#[test]
fn exact_pair_ready_payload_changes_match_buffered_updates_and_reuse_unchanged_rows() {
    let mut fixture = ReadyFixture::new();
    let [leaf, parent, root, peer, spare] = fixture.ids;
    let before = fixture.manager.readiness_generation();
    fixture
        .manager
        .commit(
            ResourceMutationBatch::new()
                .store_payload(
                    parent,
                    before.row(parent).unwrap().record.revision,
                    Arc::new(TestPayload("nested")),
                )
                .store_payload(leaf, before.row(leaf).unwrap().record.revision, 42_u64),
        )
        .expect("replace two real payload types");
    fixture.assert_parity([parent, leaf]);
    let after = fixture.manager.readiness_generation();
    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(
        after.diagnostics().publication_count,
        before.diagnostics().publication_count + 1
    );
    assert_eq!(
        after.row(leaf).unwrap().typed_load_state::<u64>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        after.row(leaf).unwrap().typed_load_state::<TestPayload>(),
        ResourceReadinessState::NotLoaded
    );
    assert_eq!(
        after
            .row(parent)
            .unwrap()
            .typed_load_state::<Arc<TestPayload>>(),
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        before.row(leaf).unwrap().typed_load_state::<TestPayload>(),
        ResourceReadinessState::Loaded
    );
    for id in [root, peer, spare] {
        assert!(Arc::ptr_eq(before.row(id).unwrap(), after.row(id).unwrap()));
    }
    fixture.assert_loaded();
    fixture.assert_unchanged_refresh([leaf, leaf]);
    fixture.assert_unchanged_refresh([parent, leaf]);
}

#[test]
fn exact_pair_failed_dependency_and_retry_ready_match_buffered_updates() {
    let mut fixture = ReadyFixture::new();
    let [leaf, parent, root, peer, spare] = fixture.ids;
    let before = fixture.manager.readiness_generation();
    let retained_payload = fixture.manager.get_untyped(leaf).unwrap();
    fixture.reload(&[peer, leaf], false);
    let reloading = fixture.manager.readiness_generation();
    fixture.reload(&[leaf, peer], true);
    let failed = fixture.manager.readiness_generation();
    assert_dependency_failure(&failed, fixture.ids);
    assert_eq!(
        fixture.manager.runtime_state(leaf),
        Some(RuntimeResourceState::Error)
    );
    assert!(Arc::ptr_eq(
        &retained_payload,
        &fixture.manager.get_untyped(leaf).unwrap()
    ));
    assert_eq!(
        failed.diagnostics().publication_count,
        reloading.diagnostics().publication_count + 1
    );
    assert_eq!(
        before.row(root).unwrap().recursive_dependency_state,
        ResourceReadinessState::Loaded
    );
    assert_eq!(
        failed.row(parent).unwrap().dependency_revision,
        reloading.row(parent).unwrap().dependency_revision + 1
    );
    assert!(Arc::ptr_eq(
        before.row(spare).unwrap(),
        failed.row(spare).unwrap()
    ));
    fixture.assert_unchanged_refresh([peer, leaf]);
    fixture.assert_unchanged_refresh([leaf, leaf]);
    fixture.retry_ready(&[peer, leaf]);
    assert_dependency_failure(&failed, fixture.ids);
    assert!(!Arc::ptr_eq(
        &failed,
        &fixture.manager.readiness_generation()
    ));
}

#[test]
fn streaming_many_ready_failed_retry_preserve_duplicate_refresh_and_caller_order() {
    let mut fixture = ReadyFixture::new();
    let [leaf, parent, root, peer, spare] = fixture.ids;
    let before = fixture.manager.readiness_generation();
    fixture.reload(&[spare, leaf, peer], false);
    fixture.reload(&[peer, spare, leaf], true);
    let failed = fixture.manager.readiness_generation();
    assert_dependency_failure(&failed, fixture.ids);
    for id in [leaf, peer, spare] {
        assert_eq!(
            fixture.manager.runtime_state(id),
            Some(RuntimeResourceState::Error)
        );
        assert!(fixture.manager.get_untyped(id).is_some());
    }
    assert_eq!(
        before.row(root).unwrap().recursive_dependency_state,
        ResourceReadinessState::Loaded
    );
    fixture.assert_unchanged_refresh([spare, root, leaf, peer, parent, leaf, spare]);
    fixture.retry_ready(&[spare, peer, leaf]);
    assert_dependency_failure(&failed, fixture.ids);
    fixture.assert_unchanged_refresh([root, spare, parent, leaf, peer, parent, root]);

    let published = fixture.manager.readiness_generation();
    let caller_ids = [root, leaf, spare, parent, peer, leaf, root];
    let records = caller_ids.map(|id| fixture.manager.registry().get(id).unwrap().clone());
    let handles = fixture
        .manager
        .register_lazy_records(records)
        .expect("unchanged ready catalog");
    assert_eq!(
        handles.iter().map(|handle| handle.id()).collect::<Vec<_>>(),
        caller_ids
    );
    fixture.assert_parity(caller_ids);
    assert!(Arc::ptr_eq(
        &published,
        &fixture.manager.readiness_generation()
    ));
    fixture.assert_loaded();
}
