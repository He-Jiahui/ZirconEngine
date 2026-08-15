use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::resource::{
    event_stream::ResourceEventPublisher, ResourceData, ResourceEventReceiver,
    ResourceEventStreamDiagnostics, ResourceId, ResourceManagementGeneration,
    ResourceReadinessGeneration, ResourceRegistry, RuntimeResourceState,
};

use super::management_projection::ResourceManagementProjection;
use super::readiness_projection::{ResourceReadinessProjection, ResourceReadinessSourceUpdate};
use super::runtime_slot::ResourceRuntimeSlot;

pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;
pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;

#[derive(Debug)]
pub(super) struct ResourceAuthority {
    pub(super) registry: ResourceRegistry,
    pub(super) management: ResourceManagementProjection,
    pub(super) payloads: ResourcePayloadMap,
    pub(super) runtime: ResourceRuntimeMap,
    pub(super) readiness: ResourceReadinessProjection,
    next_residency_token: u64,
}

impl Default for ResourceAuthority {
    fn default() -> Self {
        Self {
            registry: ResourceRegistry::default(),
            management: ResourceManagementProjection::default(),
            payloads: ResourcePayloadMap::default(),
            runtime: ResourceRuntimeMap::default(),
            readiness: ResourceReadinessProjection::default(),
            next_residency_token: 1,
        }
    }
}

impl ResourceAuthority {
    pub(super) fn allocate_residency_token(&mut self) -> u64 {
        let token = self.next_residency_token.max(1);
        self.next_residency_token = token.wrapping_add(1).max(1);
        token
    }

    pub(super) fn refresh_readiness_many(&mut self, ids: impl IntoIterator<Item = ResourceId>) {
        let mut ids = ids.into_iter().collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return;
        }

        let updates = ids
            .into_iter()
            .map(|id| ResourceReadinessSourceUpdate {
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
            })
            .collect::<Vec<_>>();
        self.readiness.apply_updates(updates);
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

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    pub(super) authority: Arc<RwLock<ResourceAuthority>>,
    commit_serial: Arc<Mutex<()>>,
    events: ResourceEventPublisher,
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

    pub(super) fn publish_event(&self, event: crate::core::resource::ResourceEvent) {
        self.events.publish(event);
    }

    #[cfg(test)]
    fn poison_event_stream_for_test(&self) {
        self.events.poison_state();
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::time::Duration;

    use crate::core::resource::{
        ModelMarker, ResourceEventKind, ResourceHandle, ResourceId, ResourceKind, ResourceLocator,
        ResourceRecord, RuntimeResourceState,
    };

    use super::ResourceManager;

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
}
