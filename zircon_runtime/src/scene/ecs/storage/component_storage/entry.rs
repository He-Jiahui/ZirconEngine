use std::any::{Any, TypeId};

use crate::scene::ecs::{ComponentId, ComponentTicks, InternalEntity, StorageType};

pub(crate) type StoredComponent = Box<dyn Any + Send + Sync>;

pub(in crate::scene::ecs::storage) struct RawRemoveResult {
    pub(super) value: StoredComponent,
    pub(super) ticks: ComponentTicks,
}

/// An erased row detached from an isolated component store before a target
/// World has assigned its own component and entity identities.
pub(crate) struct TransferredComponentRow {
    pub(super) component_id: ComponentId,
    pub(super) storage_type: StorageType,
    pub(super) type_id: TypeId,
    pub(super) source_ticks: ComponentTicks,
    pub(super) value: StoredComponent,
}

impl TransferredComponentRow {
    pub(crate) const fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub(crate) const fn source_ticks(&self) -> ComponentTicks {
        self.source_ticks
    }
}

/// A target-identity-bound row whose representation and Rust type were checked
/// against the target store before any entity row is published.
pub(crate) struct PreflightedTransferredComponentRow {
    pub(super) component_id: ComponentId,
    pub(super) row: TransferredComponentRow,
}
