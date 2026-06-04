use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsQueryFilter {
    pub collision_mask: Option<u32>,
    pub include_sensors: bool,
    #[serde(default)]
    pub excluded_entities: Vec<EntityId>,
    #[serde(default)]
    pub required_collision_group: Option<u32>,
}

impl Default for PhysicsQueryFilter {
    fn default() -> Self {
        Self {
            collision_mask: None,
            include_sensors: false,
            excluded_entities: Vec::new(),
            required_collision_group: None,
        }
    }
}
