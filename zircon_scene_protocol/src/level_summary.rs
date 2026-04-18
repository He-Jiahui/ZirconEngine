use serde::{Deserialize, Serialize};

use crate::WorldHandle;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSummary {
    pub handle: WorldHandle,
    pub entity_count: usize,
    pub selected_entity: Option<u64>,
    pub active_camera: Option<u64>,
}
