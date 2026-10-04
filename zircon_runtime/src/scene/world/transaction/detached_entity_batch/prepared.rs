use crate::scene::EntityId;

use super::DetachPreparationOwner;

/// Move-only subtree preflight bound to one World and its observable generation.
/// Dropping a preparation leaves entity rows and lifecycle state untouched.
#[derive(Debug)]
pub struct PreparedEntitySubtrees {
    pub(super) owner: DetachPreparationOwner,
    pub(super) world_generation: u64,
    pub(super) normalized_roots: Vec<EntityId>,
    pub(super) entities: Vec<EntityId>,
    pub(super) detach_preorder: Vec<EntityId>,
    pub(super) restore_order: Vec<usize>,
    pub(super) world_camera_count: usize,
    pub(super) affected_camera_count: usize,
    pub(super) hierarchy_index_rebuild_rows: usize,
}

impl PreparedEntitySubtrees {
    pub fn normalized_roots(&self) -> &[EntityId] {
        &self.normalized_roots
    }

    pub fn affected_entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn affected_camera_count(&self) -> usize {
        self.affected_camera_count
    }

    pub fn world_camera_count(&self) -> usize {
        self.world_camera_count
    }

    pub fn world_generation(&self) -> u64 {
        self.world_generation
    }
}
