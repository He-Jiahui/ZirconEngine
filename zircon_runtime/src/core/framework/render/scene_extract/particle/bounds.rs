use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleBoundsSnapshot {
    pub entity: EntityId,
    pub center: Vec3,
    pub radius: Real,
}
