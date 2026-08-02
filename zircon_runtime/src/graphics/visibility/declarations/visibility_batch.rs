use crate::core::framework::scene::EntityId;

use super::visibility_batch_key::VisibilityBatchKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityBatch {
    pub key: VisibilityBatchKey,
    /// Render-instance keys consumed by draw planning. This list is always aligned with
    /// `entities`, which retains the authoring owner for diagnostics.
    pub stable_instance_keys: Vec<u64>,
    pub entities: Vec<EntityId>,
}
