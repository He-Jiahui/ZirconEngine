use crate::core::framework::scene::EntityId;

use super::{visibility_batch_key::VisibilityBatchKey, visibility_bounds::VisibilityBounds};

#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityBvhInstance {
    pub entity: EntityId,
    pub stable_instance_key: u64,
    pub key: VisibilityBatchKey,
    pub bounds: VisibilityBounds,
}
