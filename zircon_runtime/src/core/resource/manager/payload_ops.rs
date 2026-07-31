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
        let (event_kind, id, kind, locator, revision) = {
            let mut registry = self.lock_registry_write();
            if let Some(current) = registry.get(record.id) {
                if current.state == ResourceState::Error {
                    return UntypedResourceHandle::new(current.id, current.kind);
                }
            }
            record.state = ResourceState::Ready;
            let event_kind = match registry.get(record.id) {
                Some(previous) => {
                    record.revision = next_ready_revision(previous, &record);
                    (record.revision != previous.revision).then_some(ResourceEventKind::Updated)
                }
                None => {
                    record.revision = 1;
                    Some(ResourceEventKind::Added)
                }
            };
            let id = record.id;
            let kind = record.kind;
            let revision = record.revision;
            let locator = event_kind.as_ref().map(|_| record.primary_locator.clone());
            let mut payloads = self.lock_payloads_write();
            registry.upsert(record);
            payloads.insert(id, Arc::new(payload));
            (event_kind, id, kind, locator, revision)
        };
        self.mark_runtime_loaded(id);

        if let Some(event_kind) = event_kind {
            self.broadcast(ResourceEvent {
                kind: event_kind,
                resource_kind: kind,
                id,
                locator,
                previous_locator: None,
                revision,
            });
        }

        UntypedResourceHandle::new(id, kind)
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
        let registry = self.lock_registry_read();
        let payloads = self.lock_payloads_read();
        if registry.get(handle.id())?.kind != TMarker::KIND {
            return None;
        }
        let payload = payloads.get(&handle.id())?.clone();
        Arc::downcast::<TData>(payload.into_any_arc()).ok()
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
