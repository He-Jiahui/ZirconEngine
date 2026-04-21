use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;
use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRayCastHit {
    pub entity: EntityId,
    pub distance: Real,
    pub position: [Real; 3],
    pub normal: [Real; 3],
}
