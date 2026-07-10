use std::sync::Arc;

use crate::core::resource::{
    ResourceData, ResourceEvent, ResourceEventKind, ResourceHandle, ResourceId, ResourceMarker,
    ResourceRecord, ResourceSnapshot, ResourceState, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;
use super::revision::next_ready_revision;

impl ResourceManager {
    pub fn register_ready<TData>(
        &self,
        mut record: ResourceRecord,
        payload: TData,
    ) -> UntypedResourceHandle
    where
        TData: ResourceData,
    {
        let (event_kind, record) = {
            let mut registry = self.lock_registry_write();
            let mut payloads = self.lock_payloads_write();
            let previous = registry.get(record.id).cloned();
            if previous
                .as_ref()
                .is_some_and(|current| current.state == ResourceState::Error)
            {
                let current = previous.expect("checked above");
                return UntypedResourceHandle::new(current.id, current.kind);
            }
            record.state = ResourceState::Ready;
            record.revision = previous
                .as_ref()
                .map_or(1, |current| next_ready_revision(current, &record));
            registry.upsert(record.clone());
            payloads.insert(record.id, Arc::new(payload));
            let event_kind = match previous {
                Some(previous) => {
                    if next_ready_revision(&previous, &record) != previous.revision {
                        Some(ResourceEventKind::Updated)
                    } else {
                        None
                    }
                }
                None => Some(ResourceEventKind::Added),
            };
            (event_kind, record)
        };
        self.mark_runtime_loaded(record.id);

        if let Some(event_kind) = event_kind {
            self.broadcast(ResourceEvent {
                kind: event_kind,
                resource_kind: record.kind,
                id: record.id,
                locator: Some(record.primary_locator.clone()),
                previous_locator: None,
                revision: record.revision,
            });
        }

        UntypedResourceHandle::new(record.id, record.kind)
    }

    pub fn get_untyped(&self, id: ResourceId) -> Option<Arc<dyn ResourceData>> {
        self.lock_payloads_read().get(&id).cloned()
    }

    pub fn store_payload<TData>(&self, id: ResourceId, payload: TData) -> bool
    where
        TData: ResourceData,
    {
        let registry = self.lock_registry_read();
        if registry.get(id).is_none() {
            return false;
        }
        self.lock_payloads_write().insert(id, Arc::new(payload));
        drop(registry);
        self.mark_runtime_loaded(id);
        true
    }

    pub fn get<TMarker, TData>(&self, handle: ResourceHandle<TMarker>) -> Option<Arc<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        self.snapshot::<TMarker, TData>(handle)
            .map(|snapshot| Arc::clone(snapshot.resource()))
    }

    pub fn snapshot<TMarker, TData>(
        &self,
        handle: ResourceHandle<TMarker>,
    ) -> Option<ResourceSnapshot<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let registry = self.lock_registry_read();
        let payloads = self.lock_payloads_read();
        let record = registry.get(handle.id())?.clone();
        if record.kind != TMarker::KIND {
            return None;
        }
        let payload = payloads.get(&handle.id())?.clone();
        let resource = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
        Some(ResourceSnapshot::new(record, resource))
    }
}
