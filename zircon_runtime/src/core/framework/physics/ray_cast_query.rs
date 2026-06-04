use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::PhysicsQueryFilter;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsRayCastQuery {
    pub world: WorldHandle,
    pub origin: [Real; 3],
    pub direction: [Real; 3],
    pub max_distance: Real,
    pub filter: PhysicsQueryFilter,
}
