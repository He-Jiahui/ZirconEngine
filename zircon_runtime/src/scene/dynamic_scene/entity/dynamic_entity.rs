use serde::{Deserialize, Serialize};

use crate::scene::components::NodeRecord;
use crate::scene::dynamic_scene::entity::DynamicComponent;
use crate::scene::EntityId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicEntity {
    pub source_entity: EntityId,
    pub record: NodeRecord,
    #[serde(default)]
    pub components: Vec<DynamicComponent>,
}

impl DynamicEntity {
    pub fn new(
        source_entity: EntityId,
        record: NodeRecord,
        components: Vec<DynamicComponent>,
    ) -> Self {
        Self {
            source_entity,
            record,
            components,
        }
    }
}
