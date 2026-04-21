use serde::{Deserialize, Serialize};

use super::PhysicsCombineRule;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialMetadata {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
    pub friction_combine: PhysicsCombineRule,
    pub restitution_combine: PhysicsCombineRule,
}
