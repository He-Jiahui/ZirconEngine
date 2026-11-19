//! Level-owned script activation receipts and the identity of their World incarnation.

use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Weak,
};

use super::{world::DynamicComponentCreationId, EntityId, InternalEntity};

// Each array retains its original slot numbers, including disabled bindings.
// The Level owns only start flags and row identities, never VM callback handles.
pub(crate) type ScriptBindingActivationRows = BTreeMap<
    EntityId,
    (
        (InternalEntity, DynamicComponentCreationId, u64),
        Arc<[AtomicBool]>,
    ),
>;

#[derive(Debug)]
pub(crate) struct ScriptBindingProjectionOwner {
    epoch_owner: Weak<AtomicU64>,
    replacement_epoch: u64,
}

impl ScriptBindingProjectionOwner {
    // Sampling and comparison take place while the Level's World guard is held.
    pub(super) fn capture(epoch: &Arc<AtomicU64>) -> Self {
        Self {
            epoch_owner: Arc::downgrade(epoch),
            replacement_epoch: epoch.load(Ordering::Acquire),
        }
    }

    pub(super) fn matches(&self, epoch: &Arc<AtomicU64>) -> bool {
        // The Weak pins allocation identity, not the Level. No Arc clone is needed on hits.
        std::ptr::eq(self.epoch_owner.as_ptr(), Arc::as_ptr(epoch))
            && self.replacement_epoch == epoch.load(Ordering::Acquire)
    }
}
