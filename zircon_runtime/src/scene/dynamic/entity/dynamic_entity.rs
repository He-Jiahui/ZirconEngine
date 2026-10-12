use serde::{Deserialize, Serialize};

use crate::scene::components::NodeRecord;
use crate::scene::dynamic_scene::entity::DynamicComponent;
use crate::scene::EntityId;

/// 保存来源实体的持久化状态；生成时以 source_entity 建立重映射，并把 record.id 改写为目标实体编号。
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
