use std::ops::Deref;
use std::sync::Arc;

use crate::core::resource::ResourceId;

pub struct ResourceLease<TData> {
    id: ResourceId,
    residency_token: u64,
    resource: Arc<TData>,
    release: Arc<dyn Fn(ResourceId, u64) + Send + Sync>,
}

impl<TData> ResourceLease<TData> {
    pub(crate) fn new(
        id: ResourceId,
        residency_token: u64,
        resource: Arc<TData>,
        release: Arc<dyn Fn(ResourceId, u64) + Send + Sync>,
    ) -> Self {
        Self {
            id,
            residency_token,
            resource,
            release,
        }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }

    pub fn resource(&self) -> &Arc<TData> {
        &self.resource
    }
}

impl<TData> Deref for ResourceLease<TData> {
    type Target = TData;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref()
    }
}

impl<TData> Drop for ResourceLease<TData> {
    fn drop(&mut self) {
        (self.release)(self.id, self.residency_token);
    }
}
