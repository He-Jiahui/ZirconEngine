use std::fmt;
use std::sync::Arc;

use crate::plugin::PluginModuleId;

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
}

impl fmt::Debug for OwnerRevocationListener {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnerRevocationListener")
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}
