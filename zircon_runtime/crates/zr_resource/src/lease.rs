use std::ops::Deref;
use std::sync::Arc;

use crate::ResourceId;

#[derive(Debug, Default)]
pub(crate) struct ResourceLeaseIdentity;

pub struct ResourceLease<TData> {
    id: ResourceId,
    lease_identity: Option<Arc<ResourceLeaseIdentity>>,
    resource: Arc<TData>,
    release: Arc<dyn Fn(ResourceId, Arc<ResourceLeaseIdentity>) + Send + Sync>,
}

impl<TData> ResourceLease<TData> {
    pub(crate) fn new(
        id: ResourceId,
        lease_identity: Arc<ResourceLeaseIdentity>,
        resource: Arc<TData>,
        release: Arc<dyn Fn(ResourceId, Arc<ResourceLeaseIdentity>) + Send + Sync>,
    ) -> Self {
        Self {
            id,
            lease_identity: Some(lease_identity),
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
        if let Some(lease_identity) = self.lease_identity.take() {
            (self.release)(self.id, lease_identity);
        }
    }
}
