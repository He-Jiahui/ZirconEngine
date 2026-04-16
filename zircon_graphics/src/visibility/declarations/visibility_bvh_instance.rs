use zircon_scene::EntityId;

use super::{visibility_batch_key::VisibilityBatchKey, visibility_bounds::VisibilityBounds};

#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityBvhInstance {
    pub entity: EntityId,
    pub key: VisibilityBatchKey,
    pub bounds: VisibilityBounds,
}
