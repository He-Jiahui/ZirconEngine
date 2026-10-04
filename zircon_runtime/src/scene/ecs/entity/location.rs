use serde::{Deserialize, Serialize};

use crate::scene::ecs::ArchetypeId;

/// 实体当前所在的原型表位置，供 World 的结构迁移与查询热路径使用。
/// 插入、移除或交换行后位置可能变化；调用方应通过 EntityRegistry 重新解析。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub table_row: usize,
}

impl EntityLocation {
    pub const fn new(archetype_id: ArchetypeId, table_row: usize) -> Self {
        Self {
            archetype_id,
            table_row,
        }
    }
}
