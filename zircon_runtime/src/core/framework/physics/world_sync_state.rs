use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;

use super::{
    PhysicsBodySyncState, PhysicsColliderSyncState, PhysicsJointSyncState, PhysicsMaterialSyncState,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldSyncState {
    pub world: WorldHandle,
    pub bodies: Vec<PhysicsBodySyncState>,
    pub colliders: Vec<PhysicsColliderSyncState>,
    pub joints: Vec<PhysicsJointSyncState>,
    pub materials: Vec<PhysicsMaterialSyncState>,
}

impl Default for PhysicsWorldSyncState {
    fn default() -> Self {
        Self {
            world: WorldHandle::new(0),
            bodies: Vec::new(),
            colliders: Vec::new(),
            joints: Vec::new(),
            materials: Vec::new(),
        }
    }
}
