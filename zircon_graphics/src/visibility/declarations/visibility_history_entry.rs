use zircon_framework::scene::EntityId;

use super::{visibility_batch_key::VisibilityBatchKey, visibility_bounds::VisibilityBounds};

#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityHistoryEntry {
    pub entity: EntityId,
    pub key: VisibilityBatchKey,
    pub bounds: VisibilityBounds,
}

