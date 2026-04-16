use std::sync::Arc;

use crate::{
    ResourceData, ResourceEvent, ResourceEventKind, ResourceHandle, ResourceId, ResourceMarker,
    ResourceRecord, ResourceState, UntypedResourceHandle,
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
        let event_kind = {
            let mut registry = self
                .registry
                .write()
                .expect("resource registry lock poisoned");
            let previous = registry.get(record.id).cloned();
            record.state = ResourceState::Ready;
            record.diagnostics.clear();
            record.revision = previous
                .as_ref()
                .map_or(1, |current| next_ready_revision(current, &record));
            registry.upsert(record.clone());
            match previous {
                Some(previous) => {
                    if next_ready_revision(&previous, &record) != previous.revision {
                        Some(ResourceEventKind::Updated)
                    } else {
                        None
                    }
                }
                None => Some(ResourceEventKind::Added),
            }
        };

        self.payloads
            .write()
            .expect("resource payload lock poisoned")
            .insert(record.id, Arc::new(payload));
        self.mark_runtime_loaded(record.id);

        if let Some(event_kind) = event_kind {
            self.broadcast(ResourceEvent {
                kind: event_kind,
                id: record.id,
                locator: Some(record.primary_locator.clone()),
                previous_locator: None,
                revision: record.revision,
            });
        }

        UntypedResourceHandle::new(record.id, record.kind)
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
}
