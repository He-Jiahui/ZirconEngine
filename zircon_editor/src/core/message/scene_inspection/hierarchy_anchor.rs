use serde::{Deserialize, Serialize};
use zircon_runtime::scene::EntityId;

/// Stable hierarchy-row address for an incremental scene inspection update.
///
/// Consumers identify a row by entity, retain its current tree anchor, and use subtree hash
/// changes to decide whether descendants need to be revisited. Display and reflected payloads
/// remain in the runtime artifact rather than being copied into every editor message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionHierarchyAnchor {
    entity: EntityId,
    parent: Option<EntityId>,
    depth: u32,
    subtree_hash: u64,
}

impl SceneInspectionHierarchyAnchor {
    pub fn new(entity: EntityId, parent: Option<EntityId>, depth: u32, subtree_hash: u64) -> Self {
        Self {
            entity,
            parent,
            depth,
            subtree_hash,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn parent(&self) -> Option<EntityId> {
        self.parent
    }

    pub const fn depth(&self) -> u32 {
        self.depth
    }

    pub const fn subtree_hash(&self) -> u64 {
        self.subtree_hash
    }
}
