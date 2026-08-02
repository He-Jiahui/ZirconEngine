use super::visibility_bvh_update_strategy::VisibilityBvhUpdateStrategy;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityBvhUpdatePlan {
    pub strategy: VisibilityBvhUpdateStrategy,
    pub inserted_stable_instance_keys: Vec<u64>,
    pub updated_stable_instance_keys: Vec<u64>,
    pub removed_stable_instance_keys: Vec<u64>,
}
