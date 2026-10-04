use std::ops::Deref;
use std::sync::Arc;

use crate::ResourceId;

#[derive(Debug, Default)]
pub(crate) struct ResourceLeaseIdentity;

/// 管理器对某次载荷驻留的租约；最后一个当前身份的租约释放时可卸载管理器中的载荷。
/// 热重载替换后，旧租约仍保留旧载荷，但其释放不会卸载新载荷；重载失败时允许保留最后有效版本。
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

    /// 克隆此 `Arc` 只延长载荷对象生命周期，不增加管理器的驻留租约；需要驻留保证时保留租约本身。
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
