use std::collections::HashMap;

use crate::{
    ResourceId, ResourceLocator, ResourceProjectionSnapshot, ResourceRecord, UntypedResourceHandle,
};

#[derive(Clone, Debug)]
pub struct ResourceMutationReceipt {
    records: HashMap<ResourceId, ResourceRecord>,
    removed: HashMap<ResourceId, ResourceRecord>,
    projections: ResourceProjectionSnapshot,
    published_event_count: usize,
}

impl ResourceMutationReceipt {
    pub(crate) fn new(
        records: HashMap<ResourceId, ResourceRecord>,
        removed: HashMap<ResourceId, ResourceRecord>,
        projections: ResourceProjectionSnapshot,
        published_event_count: usize,
    ) -> Self {
        Self {
            records,
            removed,
            projections,
            published_event_count,
        }
    }

    pub fn record(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.records.get(&id)
    }

    pub fn removed(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.removed.get(&id)
    }

    pub fn record_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.records
            .values()
            .find(|record| &record.primary_locator == locator)
    }

    pub fn removed_records(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.removed.values()
    }

    pub fn handle(&self, id: ResourceId) -> Option<UntypedResourceHandle> {
        self.records
            .get(&id)
            .map(|record| UntypedResourceHandle::new(id, record.kind))
    }

    pub fn projection_snapshot(&self) -> &ResourceProjectionSnapshot {
        &self.projections
    }

    pub fn published_event_count(&self) -> usize {
        self.published_event_count
    }
}
