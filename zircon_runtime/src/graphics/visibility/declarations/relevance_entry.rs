use crate::core::framework::render::PrimitiveRelevance;
use crate::core::framework::scene::EntityId;

/// 将场景实体和稳定实例键关联到每视图 render layer 资格，供 culling 与视图统计使用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityRelevanceEntry {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub relevance: PrimitiveRelevance,
}
