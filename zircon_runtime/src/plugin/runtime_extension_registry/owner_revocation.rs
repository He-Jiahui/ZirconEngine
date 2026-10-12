use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::plugin::PluginModuleId;

// 外部扩展家族的撤销回调随注册 owner 存活；克隆目录时共享回调所有权。
#[derive(Clone)]
pub(super) struct OwnerRevocationListener {
    owner: PluginModuleId,
    callback: Arc<dyn Fn(PluginModuleId) + Send + Sync>,
}

impl OwnerRevocationListener {
    pub(super) fn new(
        owner: PluginModuleId,
        callback: impl Fn(PluginModuleId) + Send + Sync + 'static,
    ) -> Self {
        Self {
            owner,
            callback: Arc::new(callback),
        }
    }

    pub(super) fn owner(&self) -> PluginModuleId {
        self.owner
    }

    pub(super) fn notify(&self, revoked_owner: PluginModuleId) {
        (self.callback)(revoked_owner);
    }

    pub(super) fn notify_catching_panic(&self, revoked_owner: PluginModuleId) -> bool {
        catch_unwind(AssertUnwindSafe(|| self.notify(revoked_owner))).is_ok()
    }

    pub(super) fn callback(&self) -> Arc<dyn Fn(PluginModuleId) + Send + Sync> {
        Arc::clone(&self.callback)
    }
}

impl fmt::Debug for OwnerRevocationListener {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnerRevocationListener")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}
