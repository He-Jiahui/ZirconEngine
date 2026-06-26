use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::resource::{ResourceData, ResourceEvent, ResourceId, ResourceRegistry};

use super::runtime_slot::ResourceRuntimeSlot;

pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;
pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;
pub(super) type ResourceSubscriberList = Vec<Sender<ResourceEvent>>;

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    pub(super) registry: Arc<RwLock<ResourceRegistry>>,
    pub(super) payloads: Arc<RwLock<ResourcePayloadMap>>,
    pub(super) runtime: Arc<RwLock<ResourceRuntimeMap>>,
    pub(super) subscribers: Arc<Mutex<ResourceSubscriberList>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> Receiver<ResourceEvent> {
        let (sender, receiver) = unbounded();
        self.lock_subscribers().push(sender);
        receiver
    }

    pub fn registry(&self) -> RwLockReadGuard<'_, ResourceRegistry> {
        self.lock_registry_read()
    }

    pub(super) fn lock_registry_read(&self) -> RwLockReadGuard<'_, ResourceRegistry> {
        self.registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_registry_write(&self) -> RwLockWriteGuard<'_, ResourceRegistry> {
        self.registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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

    pub(super) fn lock_subscribers(&self) -> MutexGuard<'_, ResourceSubscriberList> {
        self.subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
            let _guard = manager.lock_subscribers();
            panic!("poison resource subscribers");
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
