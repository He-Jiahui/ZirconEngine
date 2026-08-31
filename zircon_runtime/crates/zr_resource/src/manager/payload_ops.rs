use std::sync::Arc;

use crate::{
    ResourceData, ResourceHandle, ResourceId, ResourceMarker, ResourceMutationBatch,
    ResourceRecord, ResourceResult, ResourceSnapshot, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn register_ready<TData>(
        &self,
        record: ResourceRecord,
        payload: TData,
    ) -> ResourceResult<UntypedResourceHandle>
    where
        TData: ResourceData,
    {
        let id = record.id;
        let receipt = self.commit(ResourceMutationBatch::new().upsert_ready(record, payload))?;
        Ok(receipt
            .handle(id)
            .expect("a committed ready upsert produces a handle"))
    }

    pub fn get_untyped(&self, id: ResourceId) -> Option<Arc<dyn ResourceData>> {
        self.lock_authority_read().payloads.get(&id).cloned()
    }

    pub fn store_payload<TData>(
        &self,
        id: ResourceId,
        expected_revision: u64,
        payload: TData,
    ) -> ResourceResult<()>
    where
        TData: ResourceData,
    {
        self.commit(ResourceMutationBatch::new().store_payload(id, expected_revision, payload))?;
        Ok(())
    }

    pub fn get<TMarker, TData>(&self, handle: ResourceHandle<TMarker>) -> Option<Arc<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let authority = self.lock_authority_read();
        if authority.registry.get(handle.id())?.kind != TMarker::KIND {
            return None;
        }
        let payload = authority.payloads.get(&handle.id())?.clone();
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
        let authority = self.lock_authority_read();
        let record = authority.registry.get(handle.id())?.clone();
        if record.kind != TMarker::KIND {
            return None;
        }
        let payload = authority.payloads.get(&handle.id())?.clone();
        let resource = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
        Some(ResourceSnapshot::new(record, resource))
    }
}
