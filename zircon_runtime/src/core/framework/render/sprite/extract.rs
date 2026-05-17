use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderPhaseQueue;

use super::RenderSpriteSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteExtract {
    pub sprites: Vec<RenderSpriteSnapshot>,
    pub phase_queue: RenderPhaseQueue,
}
