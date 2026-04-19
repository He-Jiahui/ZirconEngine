use zircon_framework::scene::EntityId;

use super::visibility_bvh_update_strategy::VisibilityBvhUpdateStrategy;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityBvhUpdatePlan {
    pub strategy: VisibilityBvhUpdateStrategy,
    pub inserted_entities: Vec<EntityId>,
    pub updated_entities: Vec<EntityId>,
    pub removed_entities: Vec<EntityId>,
}

