use crate::core::framework::scene::EntityId;

use super::VisibilityRenderableInput;

/// 场景提取交给可见性规划的实体集合与逐实例条目。
/// 实体列表用于粗粒度分类，逐实例键防止同一实体的多个图元被合并。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct VisibilityInput {
    pub renderable_entities: Vec<EntityId>,
    pub static_entities: Vec<EntityId>,
    pub dynamic_entities: Vec<EntityId>,
    pub renderables: Vec<VisibilityRenderableInput>,
}
