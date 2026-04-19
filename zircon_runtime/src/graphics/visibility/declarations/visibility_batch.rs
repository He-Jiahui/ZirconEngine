use crate::core::framework::scene::EntityId;

use super::visibility_batch_key::VisibilityBatchKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityBatch {
    pub key: VisibilityBatchKey,
    pub entities: Vec<EntityId>,
}
