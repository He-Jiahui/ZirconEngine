use crate::scene::EntityId;

/// 对外暴露稳定场景实体 ID 的语义约束；跨 World 或跨替换代使用时仍须由当前 World 校验存在性。
pub trait EntityIdentity: Copy + Eq + Send + Sync {
    fn entity_id(self) -> EntityId;
}

/// 标记可跨线程持有的静态组件数据角色；World 的存储资格由独立的 ECS Component 契约决定。
pub trait ComponentData: Send + Sync + 'static {}

impl EntityIdentity for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}
