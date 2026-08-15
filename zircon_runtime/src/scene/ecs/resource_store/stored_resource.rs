use std::any::{Any, TypeId};

use crate::scene::ecs::ComponentTicks;

pub(super) struct StoredResource {
    pub(super) value: Box<dyn Any + Send + Sync>,
    pub(super) type_name: &'static str,
    pub(super) ticks: ComponentTicks,
}

/// An owned resource value detached from an isolated mutation artifact. The
/// target store rebases its ticks when publication begins.
pub(crate) struct TransferredResourceRow {
    pub(super) type_id: TypeId,
    pub(super) value: Box<dyn Any + Send + Sync>,
    pub(super) type_name: &'static str,
    pub(super) source_ticks: ComponentTicks,
}

impl TransferredResourceRow {
    pub(crate) const fn source_ticks(&self) -> ComponentTicks {
        self.source_ticks
    }
}
