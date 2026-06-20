use crate::core::framework::scene::EntityId;
use serde::{Deserialize, Serialize};

use super::{RenderPhase, RenderPhaseSortKey};

/// Public diagnostic form of the exact tuple used to order phase queue items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RenderPhaseQueueOrderingKey {
    pub phase_order: u8,
    pub sort_key: RenderPhaseSortKey,
    pub entity: EntityId,
}

impl RenderPhaseQueueOrderingKey {
    pub const fn new(phase: RenderPhase, sort_key: RenderPhaseSortKey, entity: EntityId) -> Self {
        Self {
            phase_order: phase.queue_order(),
            sort_key,
            entity,
        }
    }

    pub const fn raw_sort_key(self) -> u64 {
        self.sort_key.raw()
    }
}
