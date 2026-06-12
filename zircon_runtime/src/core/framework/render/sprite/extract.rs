use serde::{Deserialize, Serialize};

use crate::core::framework::render::{RenderPhaseQueue, RenderPhaseQueueSummary};

use super::RenderSpriteSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteExtract {
    pub sprites: Vec<RenderSpriteSnapshot>,
    pub phase_queue: RenderPhaseQueue,
}

impl SpriteExtract {
    /// Builds a diagnostics summary from the current sorted sprite phase queue.
    pub fn phase_queue_summary(&self) -> RenderPhaseQueueSummary {
        self.phase_queue.summary()
    }
}
