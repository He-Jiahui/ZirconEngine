use std::sync::Arc;

use crate::{RuntimeResourceState, lease::ResourceLeaseIdentity};

#[derive(Debug)]
pub(super) struct ResourceRuntimeSlot {
    pub(super) lease_identity: Arc<ResourceLeaseIdentity>,
    pub(super) state: RuntimeResourceState,
}

impl Default for ResourceRuntimeSlot {
    fn default() -> Self {
        Self {
            lease_identity: Arc::new(ResourceLeaseIdentity),
            state: RuntimeResourceState::default(),
        }
    }
}
