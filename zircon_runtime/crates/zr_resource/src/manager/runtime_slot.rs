use crate::{lease::ResourceLeaseIdentity, ResourceActiveVersionKey, RuntimeResourceState};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct PreparedResourceReservation {
    pub(super) token: Arc<()>,
    pub(super) key: ResourceActiveVersionKey,
    pub(super) deferred_last_release: bool,
}

#[derive(Debug)]
pub(super) struct ResourceRuntimeSlot {
    pub(super) lease_identity: Arc<ResourceLeaseIdentity>,
    pub(super) state: RuntimeResourceState,
    pub(super) retain_last_good: bool,
    pub(super) prepared: Option<PreparedResourceReservation>,
}

impl Default for ResourceRuntimeSlot {
    fn default() -> Self {
        Self {
            lease_identity: Arc::new(ResourceLeaseIdentity),
            state: RuntimeResourceState::default(),
            retain_last_good: false,
            prepared: None,
        }
    }
}
