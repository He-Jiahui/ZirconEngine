use crate::core::framework::scene::EntityId;

/// 视图相关探针候选及预算，供 Hybrid GI 计划与诊断共享实体身份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityHybridGiProbe {
    pub entity: EntityId,
    pub probe_id: u32,
    pub resident: bool,
    pub ray_budget: u32,
}
