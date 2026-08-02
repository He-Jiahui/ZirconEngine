use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::framework::asset::ResourceManagementGeneration;
use crate::core::resource::{
    event_stream::ResourceEventPublisher, ResourceData, ResourceEventReceiver,
    ResourceEventStreamDiagnostics, ResourceId, ResourceReadinessGeneration, ResourceRegistry,
    RuntimeResourceState,
};

use super::management_projection::ResourceManagementProjection;
use super::readiness_projection::{ResourceReadinessProjection, ResourceReadinessSourceUpdate};
use super::runtime_slot::ResourceRuntimeSlot;

pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;
pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;

#[derive(Debug, Default)]
struct ResourceAuthority {
    registry: ResourceRegistry,
    management: ResourceManagementProjection,
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

pub(super) struct ResourceAuthorityWriteGuard<'a> {
    guard: RwLockWriteGuard<'a, ResourceAuthority>,
}

impl Deref for ResourceAuthorityWriteGuard<'_> {
    type Target = ResourceRegistry;

    fn deref(&self) -> &Self::Target {
        &self.guard.registry
    }
}

impl DerefMut for ResourceAuthorityWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.registry
    }
}

impl ResourceAuthorityWriteGuard<'_> {
    pub(super) fn upsert(
        &mut self,
        record: crate::core::resource::ResourceRecord,
    ) -> Option<crate::core::resource::ResourceRecord> {
        let previous = self.guard.registry.upsert(record.clone());
        self.guard.management.upsert(&record);
        previous
    }

    pub(super) fn upsert_registry_only(
        &mut self,
        record: crate::core::resource::ResourceRecord,
    ) -> Option<crate::core::resource::ResourceRecord> {
        self.guard.registry.upsert(record)
    }

    pub(super) fn publish_records<'a>(
        &mut self,
        records: impl IntoIterator<Item = &'a crate::core::resource::ResourceRecord>,
    ) {
        self.guard.management.upsert_many(records);
    }

    pub(super) fn publish_record(&mut self, record: &crate::core::resource::ResourceRecord) {
        self.guard.management.upsert(record);
    }

    pub(super) fn remove_by_locator(
        &mut self,
        locator: &crate::core::resource::ResourceLocator,
    ) -> Option<crate::core::resource::ResourceRecord> {
        let removed = self.guard.registry.remove_by_locator(locator)?;
        self.guard.management.remove(removed.id);
        Some(removed)
    }

    pub(super) fn rename(
        &mut self,
        from: &crate::core::resource::ResourceLocator,
        to: crate::core::resource::ResourceLocator,
    ) -> crate::core::resource::ResourceResult<crate::core::resource::ResourceRecord> {
        let renamed = self.guard.registry.rename(from, to)?;
        self.guard.management.upsert(&renamed);
        Ok(renamed)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    authority: Arc<RwLock<ResourceAuthority>>,
    pub(super) payloads: Arc<RwLock<ResourcePayloadMap>>,
    pub(super) runtime: Arc<RwLock<ResourceRuntimeMap>>,
    events: ResourceEventPublisher,
    readiness: Arc<RwLock<ResourceReadinessProjection>>,
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
        self.readiness
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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

    pub(super) fn lock_registry_write(&self) -> ResourceAuthorityWriteGuard<'_> {
        ResourceAuthorityWriteGuard {
            guard: self
                .authority
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }

    pub fn registry(&self) -> ResourceRegistryReadGuard<'_> {
        self.lock_registry_read()
    }

    pub(super) fn lock_payloads_read(&self) -> RwLockReadGuard<'_, ResourcePayloadMap> {
        self.payloads
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_payloads_write(&self) -> RwLockWriteGuard<'_, ResourcePayloadMap> {
        self.payloads
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_runtime_read(&self) -> RwLockReadGuard<'_, ResourceRuntimeMap> {
        self.runtime
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_runtime_write(&self) -> RwLockWriteGuard<'_, ResourceRuntimeMap> {
        self.runtime
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn publish_event(&self, event: crate::core::resource::ResourceEvent) {
        self.events.publish(event);
    }

    pub(super) fn refresh_readiness(&self, id: ResourceId) {
        self.refresh_readiness_many(std::iter::once(id));
    }

    pub(super) fn refresh_readiness_many(&self, ids: impl IntoIterator<Item = ResourceId>) {
        let mut ids = ids.into_iter().collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return;
        }

        let registry = self.lock_registry_read();
        let runtime = self.lock_runtime_read();
        let payloads = self.lock_payloads_read();
        let updates = ids
            .into_iter()
            .map(|id| ResourceReadinessSourceUpdate {
                id,
                record: registry.get(id).cloned(),
                runtime_state: runtime
                    .get(&id)
                    .map(|slot| slot.state)
                    .unwrap_or(RuntimeResourceState::Unloaded),
                payload_type_id: payloads.get(&id).map(|payload| payload.as_any().type_id()),
            })
            .collect::<Vec<_>>();
        drop(payloads);
        drop(runtime);
        drop(registry);

        self.readiness
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .apply_updates(updates);
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
            let _guard = manager.lock_registry_write();
            panic!("poison resource registry");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.lock_payloads_write();
            panic!("poison resource payloads");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.lock_runtime_write();
            panic!("poison resource runtime slots");
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
