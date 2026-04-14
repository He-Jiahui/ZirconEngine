use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::{
    ResourceData, ResourceDiagnostic, ResourceEvent, ResourceEventKind, ResourceHandle,
    ResourceId, ResourceLease, ResourceLocator, ResourceMarker, ResourceRecord,
    ResourceRegistry, ResourceState, RuntimeResourceState, UntypedResourceHandle,
};

#[derive(Clone, Debug, Default)]
struct ResourceRuntimeSlot {
    ref_count: usize,
    state: RuntimeResourceState,
}

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    registry: Arc<RwLock<ResourceRegistry>>,
    payloads: Arc<RwLock<HashMap<ResourceId, Arc<dyn ResourceData>>>>,
    runtime: Arc<RwLock<HashMap<ResourceId, ResourceRuntimeSlot>>>,
    subscribers: Arc<Mutex<Vec<Sender<ResourceEvent>>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> Receiver<ResourceEvent> {
        let (sender, receiver) = unbounded();
        self.subscribers
            .lock()
            .expect("resource subscribers lock poisoned")
            .push(sender);
        receiver
    }

    pub fn registry(&self) -> std::sync::RwLockReadGuard<'_, ResourceRegistry> {
        self.registry.read().expect("resource registry lock poisoned")
    }

    pub fn register_record(&self, record: ResourceRecord) -> UntypedResourceHandle {
        let event_kind = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            match registry.upsert(record.clone()) {
                Some(_) => ResourceEventKind::Updated,
                None => ResourceEventKind::Added,
            }
        };
        self.ensure_runtime_slot(record.id);

        self.broadcast(ResourceEvent {
            kind: event_kind,
            id: record.id,
            locator: Some(record.primary_locator.clone()),
            previous_locator: None,
            revision: record.revision,
        });

        UntypedResourceHandle::new(record.id, record.kind)
    }

    pub fn register_ready<TData>(
        &self,
        mut record: ResourceRecord,
        payload: TData,
    ) -> UntypedResourceHandle
    where
        TData: ResourceData,
    {
        let event_kind = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let previous = registry.get(record.id).cloned();
            record.state = ResourceState::Ready;
            record.diagnostics.clear();
            record.revision = previous.as_ref().map_or(1, |current| current.revision + 1);
            let kind = if previous.is_some() {
                ResourceEventKind::Updated
            } else {
                ResourceEventKind::Added
            };
            registry.upsert(record.clone());
            kind
        };

        self.payloads
            .write()
            .expect("resource payload lock poisoned")
            .insert(record.id, Arc::new(payload));
        self.mark_runtime_loaded(record.id);

        self.broadcast(ResourceEvent {
            kind: event_kind,
            id: record.id,
            locator: Some(record.primary_locator.clone()),
            previous_locator: None,
            revision: record.revision,
        });

        UntypedResourceHandle::new(record.id, record.kind)
    }

    pub fn start_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<ResourceRecord> {
        let updated = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let mut record = registry.get(id).cloned()?;
            record.state = ResourceState::Reloading;
            record.diagnostics = diagnostics;
            registry.upsert(record.clone());
            record
        };
        self.set_runtime_state(id, RuntimeResourceState::Reloading);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Updated,
            id,
            locator: Some(updated.primary_locator.clone()),
            previous_locator: None,
            revision: updated.revision,
        });

        Some(updated)
    }

    pub fn fail_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> Option<ResourceRecord> {
        let updated = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let mut record = registry.get(id).cloned()?;
            record.state = ResourceState::Error;
            record.diagnostics = diagnostics;
            registry.upsert(record.clone());
            record
        };
        self.set_runtime_state(id, RuntimeResourceState::Error);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::ReloadFailed,
            id,
            locator: Some(updated.primary_locator.clone()),
            previous_locator: None,
            revision: updated.revision,
        });

        Some(updated)
    }

    pub fn remove_by_locator(&self, locator: &ResourceLocator) -> Option<ResourceRecord> {
        let removed = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            registry.remove_by_locator(locator)?
        };

        self.payloads
            .write()
            .expect("resource payload lock poisoned")
            .remove(&removed.id);
        self.runtime
            .write()
            .expect("resource runtime lock poisoned")
            .remove(&removed.id);

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Removed,
            id: removed.id,
            locator: Some(removed.primary_locator.clone()),
            previous_locator: None,
            revision: removed.revision,
        });

        Some(removed)
    }

    pub fn rename(
        &self,
        from: &ResourceLocator,
        to: ResourceLocator,
    ) -> Result<ResourceRecord, String> {
        let renamed = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            registry.rename(from, to.clone())?
        };

        self.broadcast(ResourceEvent {
            kind: ResourceEventKind::Renamed,
            id: renamed.id,
            locator: Some(renamed.primary_locator.clone()),
            previous_locator: Some(from.clone()),
            revision: renamed.revision,
        });

        Ok(renamed)
    }

    pub fn get_untyped(&self, id: ResourceId) -> Option<Arc<dyn ResourceData>> {
        self.payloads
            .read()
            .expect("resource payload lock poisoned")
            .get(&id)
            .cloned()
    }

    pub fn store_payload<TData>(&self, id: ResourceId, payload: TData) -> bool
    where
        TData: ResourceData,
    {
        if self.registry().get(id).is_none() {
            return false;
        }
        self.payloads
            .write()
            .expect("resource payload lock poisoned")
            .insert(id, Arc::new(payload));
        self.mark_runtime_loaded(id);
        true
    }

    pub fn acquire<TMarker, TData>(
        &self,
        handle: ResourceHandle<TMarker>,
    ) -> Option<ResourceLease<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let record = self.registry().get(handle.id()).cloned()?;
        if record.kind != TMarker::KIND {
            return None;
        }

        let payload = self.get_untyped(handle.id())?;
        let payload = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
        {
            let mut runtime = self.runtime.write().expect("resource runtime lock poisoned");
            let slot = runtime.entry(handle.id()).or_default();
            slot.ref_count += 1;
            slot.state = RuntimeResourceState::Loaded;
        }

        let manager = self.clone();
        Some(ResourceLease::new(
            handle.id(),
            payload,
            Arc::new(move |id| {
                let _ = manager.release(id);
            }),
        ))
    }

    pub fn release(&self, id: ResourceId) -> Option<usize> {
        let next_ref_count = {
            let mut runtime = self.runtime.write().expect("resource runtime lock poisoned");
            let slot = runtime.get_mut(&id)?;
            if slot.ref_count == 0 {
                return Some(0);
            }
            slot.ref_count -= 1;
            if slot.ref_count == 0 {
                slot.state = RuntimeResourceState::Unloaded;
            }
            slot.ref_count
        };

        if next_ref_count == 0 {
            self.payloads
                .write()
                .expect("resource payload lock poisoned")
                .remove(&id);
        }

        Some(next_ref_count)
    }

    pub fn ref_count(&self, id: ResourceId) -> Option<usize> {
        self.runtime
            .read()
            .expect("resource runtime lock poisoned")
            .get(&id)
            .map(|slot| slot.ref_count)
            .or_else(|| self.registry().get(id).map(|_| 0))
    }

    pub fn runtime_state(&self, id: ResourceId) -> Option<RuntimeResourceState> {
        self.runtime
            .read()
            .expect("resource runtime lock poisoned")
            .get(&id)
            .map(|slot| slot.state)
            .or_else(|| {
                self.registry()
                    .get(id)
                    .map(|_| RuntimeResourceState::Unloaded)
            })
    }

    pub fn get<TMarker, TData>(&self, handle: ResourceHandle<TMarker>) -> Option<Arc<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let record = self.registry().get(handle.id()).cloned()?;
        if record.kind != TMarker::KIND {
            return None;
        }
        let payload = self.get_untyped(handle.id())?;
        Arc::downcast::<TData>(payload.into_any_arc()).ok()
    }

    fn broadcast(&self, event: ResourceEvent) {
        let mut subscribers = self
            .subscribers
            .lock()
            .expect("resource subscribers lock poisoned");
        subscribers.retain(|sender| sender.send(event.clone()).is_ok());
    }

    fn ensure_runtime_slot(&self, id: ResourceId) {
        let has_payload = self.get_untyped(id).is_some();
        let mut runtime = self.runtime.write().expect("resource runtime lock poisoned");
        runtime.entry(id).or_insert_with(|| ResourceRuntimeSlot {
            ref_count: 0,
            state: if has_payload {
                RuntimeResourceState::Loaded
            } else {
                RuntimeResourceState::Unloaded
            },
        });
    }

    fn mark_runtime_loaded(&self, id: ResourceId) {
        let mut runtime = self.runtime.write().expect("resource runtime lock poisoned");
        let slot = runtime.entry(id).or_default();
        slot.state = RuntimeResourceState::Loaded;
    }

    fn set_runtime_state(&self, id: ResourceId, state: RuntimeResourceState) {
        let mut runtime = self.runtime.write().expect("resource runtime lock poisoned");
        let slot = runtime.entry(id).or_default();
        slot.state = state;
    }
}
