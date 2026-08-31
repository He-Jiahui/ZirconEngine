use crate::core::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleBillboardBasisSnapshot {
    pub right: Vec3,
    pub up: Vec3,
}

impl RenderParticleBillboardBasisSnapshot {
    pub const fn new(right: Vec3, up: Vec3) -> Self {
        Self { right, up }
    }
}
