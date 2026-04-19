use std::ops::Deref;
use std::sync::Arc;

use crate::core::resource::ResourceId;

pub struct ResourceLease<TData> {
    id: ResourceId,
    resource: Arc<TData>,
    release: Arc<dyn Fn(ResourceId) + Send + Sync>,
}

impl<TData> ResourceLease<TData> {
    pub fn new(
        id: ResourceId,
        resource: Arc<TData>,
        release: Arc<dyn Fn(ResourceId) + Send + Sync>,
    ) -> Self {
        Self {
            id,
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
        (self.release)(self.id);
    }
}
