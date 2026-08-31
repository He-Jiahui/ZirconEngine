use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    ResourceData, ResourceEventReceiver, ResourceEventStreamDiagnostics, ResourceId,
    ResourceManagementGeneration, ResourceManagementGenerationIdentity,
    ResourceReadinessGeneration, ResourceReadinessGenerationIdentity, ResourceRegistry,
    RuntimeResourceState, event_stream::ResourceEventPublisher,
};

use super::management_projection::ResourceManagementProjection;
use super::readiness_projection::{ResourceReadinessProjection, ResourceReadinessSourceUpdate};
use super::runtime_slot::ResourceRuntimeSlot;

pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;
pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;

#[derive(Debug, Default)]
pub(super) struct ResourceAuthority {
    pub(super) registry: ResourceRegistry,
    pub(super) management: ResourceManagementProjection,
    pub(super) payloads: ResourcePayloadMap,
    pub(super) runtime: ResourceRuntimeMap,
    pub(super) readiness: ResourceReadinessProjection,
}

impl ResourceAuthority {
    pub(super) fn refresh_readiness_many(&mut self, ids: impl IntoIterator<Item = ResourceId>) {
        let mut ids = ids.into_iter();
        let Some(first) = ids.next() else {
            return;
        };
        let Some(second) = ids.next() else {
            self.refresh_readiness_one(first);
            return;
        };

        let (remaining_lower_bound, _) = ids.size_hint();
        let mut unique_ids = Vec::with_capacity(remaining_lower_bound.saturating_add(2));
        unique_ids.push(first);
        unique_ids.push(second);
        unique_ids.extend(ids);
        unique_ids.sort_unstable();
        unique_ids.dedup();

        let updates = unique_ids
            .into_iter()
            .map(|id| self.readiness_source_update(id))
            .collect::<Vec<_>>();
        self.readiness.apply_updates(updates);
    }

    fn refresh_readiness_one(&mut self, id: ResourceId) {
        let update = self.readiness_source_update(id);
        self.readiness.apply_updates([update]);
    }

    fn readiness_source_update(&self, id: ResourceId) -> ResourceReadinessSourceUpdate {
        ResourceReadinessSourceUpdate {
            id,
            record: self.registry.get(id).cloned(),
            runtime_state: self
                .runtime
                .get(&id)
                .map(|slot| slot.state)
                .unwrap_or(RuntimeResourceState::Unloaded),
            payload_type_id: self
                .payloads
                .get(&id)
                .map(|payload| payload.as_any().type_id()),
        }
    }
}

#[derive(Debug)]
pub struct ResourceRegistryReadGuard<'a> {
    guard: RwLockReadGuard<'a, ResourceAuthority>,
}

impl Deref for ResourceRegistryReadGuard<'_> {
    type Target = ResourceRegistry;

    fn deref(&self) -> &Self::Target {
        &self.guard.registry
    }
}

/// Exact management/readiness pair captured under one Resource authority lock.
#[derive(Clone, Debug)]
pub struct ResourceProjectionSnapshot {
    management: Arc<ResourceManagementGeneration>,
    readiness: Arc<ResourceReadinessGeneration>,
}

impl ResourceProjectionSnapshot {
    pub(super) fn new(
        management: Arc<ResourceManagementGeneration>,
        readiness: Arc<ResourceReadinessGeneration>,
    ) -> Self {
        Self {
            management,
            readiness,
        }
    }

    pub fn management(&self) -> &Arc<ResourceManagementGeneration> {
        &self.management
    }

    pub fn readiness(&self) -> &Arc<ResourceReadinessGeneration> {
        &self.readiness
    }

    pub fn management_identity(&self) -> ResourceManagementGenerationIdentity {
        self.management.identity()
    }

    pub fn readiness_identity(&self) -> ResourceReadinessGenerationIdentity {
        self.readiness.identity()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    pub(super) authority: Arc<RwLock<ResourceAuthority>>,
    commit_serial: Arc<Mutex<()>>,
    pub(super) events: ResourceEventPublisher,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> ResourceEventReceiver {
        self.events.subscribe()
    }

    pub fn event_stream_diagnostics(&self) -> ResourceEventStreamDiagnostics {
        self.events.diagnostics()
    }

    pub fn projection_snapshot(&self) -> ResourceProjectionSnapshot {
        let authority = self
            .authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ResourceProjectionSnapshot::new(
            authority.management.generation(),
            authority.readiness.generation(),
        )
    }

    pub fn management_generation(&self) -> Arc<ResourceManagementGeneration> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .management
            .generation()
    }

    pub fn readiness_generation(&self) -> Arc<ResourceReadinessGeneration> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .readiness
            .generation()
    }

    pub(super) fn lock_registry_read(&self) -> ResourceRegistryReadGuard<'_> {
        ResourceRegistryReadGuard {
            guard: self
                .authority
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub(super) fn lock_authority_read(&self) -> RwLockReadGuard<'_, ResourceAuthority> {
        self.authority
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_authority_write(&self) -> RwLockWriteGuard<'_, ResourceAuthority> {
        self.authority
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_commit_serial(&self) -> MutexGuard<'_, ()> {
        self.commit_serial
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn commit_gate_is_locked_for_test(&self) -> bool {
        self.commit_serial.try_lock().is_err()
    }

    pub fn registry(&self) -> ResourceRegistryReadGuard<'_> {
        self.lock_registry_read()
    }

    #[cfg(test)]
    fn poison_event_stream_for_test(&self) {
        self.events.poison_state();
    }

    #[cfg(test)]
    pub(crate) fn set_event_next_sequence_for_test(&self, next_sequence: Option<u64>) {
        self.events.set_next_sequence_for_test(next_sequence);
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;
    use std::time::Duration;

    use crate::{
        ModelMarker, ResourceEventKind, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
        ResourceRecord, RuntimeResourceState,
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
    fn single_readiness_refresh_does_not_buffer_and_sort_ids() {
        let source = include_str!("resource_manager.rs");
        let refresh = source
            .split("pub(super) fn refresh_readiness_many")
            .nth(1)
            .and_then(|source| source.split("fn readiness_source_update").next())
            .expect("readiness refresh implementation");

        assert!(refresh.contains("let Some(second) = ids.next() else {"));
        assert!(refresh.contains("self.refresh_readiness_one(first);"));
        assert!(!refresh.contains("let mut ids = ids.into_iter().collect::<Vec<_>>();"));
    }
}
