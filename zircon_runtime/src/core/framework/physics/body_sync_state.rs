use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Transform};

use super::PhysicsBodyType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsBodySyncState {
    pub entity: EntityId,
    pub body_type: PhysicsBodyType,
    pub transform: Transform,
    pub mass: Real,
    pub linear_damping: Real,
    pub angular_damping: Real,
    pub gravity_scale: Real,
    pub can_sleep: bool,
    pub lock_translation: [bool; 3],
    pub lock_rotation: [bool; 3],
}
