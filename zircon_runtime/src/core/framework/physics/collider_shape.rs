use serde::{Deserialize, Serialize};

use crate::core::math::Real;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicsColliderShape {
    Box { half_extents: [Real; 3] },
    Sphere { radius: Real },
    Capsule { radius: Real, half_height: Real },
}
