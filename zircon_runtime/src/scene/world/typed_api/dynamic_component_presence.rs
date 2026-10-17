use std::sync::Arc;

use crate::scene::ecs::{Component, StorageType};
use zircon_runtime_interface::retained_memory::{
    ImmutableRetainedOwnerV1, RetainedBoundBuilderV1, RetainedOwnerV1, RetentionContractError,
};

/// Ephemeral proof of one dynamic component's creation, within its owning World.
/// Serialization and projection rebuilding deliberately do not preserve this proof.
#[derive(Clone, Debug)]
pub struct DynamicComponentCreationId(Arc<()>);

impl Default for DynamicComponentCreationId {
    fn default() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for DynamicComponentCreationId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DynamicComponentCreationId {}

impl RetainedOwnerV1 for DynamicComponentCreationId {
    fn retained_bound_v1(&self, meter: &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> {
        self.0.retained_bound_v1(meter)
    }
}
// The token's payload is immutable and allocation-free; aliases cannot grow it.
unsafe impl ImmutableRetainedOwnerV1 for DynamicComponentCreationId {}

impl DynamicComponentCreationId {
    /// Pure quote for a future fresh token. No Arc is allocated by this proof.
    pub fn allocation_bound_v1(meter: &mut RetainedBoundBuilderV1) -> Result<(), RetentionContractError> {
        meter.arc_payload(&())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::scene::world) struct DynamicComponentPresence {
    pub(in crate::scene::world) creation: DynamicComponentCreationId,
}

zircon_runtime_interface::impl_retained_owner_v1!(DynamicComponentPresence { creation });
impl Component for DynamicComponentPresence {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}
