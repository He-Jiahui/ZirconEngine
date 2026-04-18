use serde::{Deserialize, Serialize};

use super::{EntityId, WorldHandle};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSummary {
    pub handle: WorldHandle,
    pub entity_count: usize,
    pub selected_entity: Option<EntityId>,
    pub active_camera: Option<EntityId>,
}
