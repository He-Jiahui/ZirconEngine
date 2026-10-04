use super::visibility_bvh_update_strategy::VisibilityBvhUpdateStrategy;

/// 当前帧相对于历史快照的 BVH 改动集；空间索引与上传计划必须消费同一策略。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityBvhUpdatePlan {
    pub strategy: VisibilityBvhUpdateStrategy,
    pub inserted_stable_instance_keys: Vec<u64>,
    pub updated_stable_instance_keys: Vec<u64>,
    pub removed_stable_instance_keys: Vec<u64>,
}
