use std::ops::Deref;
use std::sync::Arc;

use crate::core::resource::ResourceRecord;

/// Immutable payload paired atomically with the exact record revision that owns it.
#[derive(Debug)]
pub struct ResourceSnapshot<TData> {
    record: ResourceRecord,
    resource: Arc<TData>,
}

impl<TData> ResourceSnapshot<TData> {
    pub fn new(record: ResourceRecord, resource: Arc<TData>) -> Self {
        Self { record, resource }
    }

    pub fn record(&self) -> &ResourceRecord {
        &self.record
    }

    pub fn revision(&self) -> u64 {
        self.record.revision
    }

    pub fn resource(&self) -> &Arc<TData> {
        &self.resource
    }
}

impl<TData> Deref for ResourceSnapshot<TData> {
    type Target = TData;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref()
    }
}
