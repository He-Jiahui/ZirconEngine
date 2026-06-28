use crate::core::math::{RenderVec3, RenderVec4, Vec3, Vec4};

use super::ParticleVertex;

impl ParticleVertex {
    pub(in crate::graphics::scene::scene_renderer::particle) fn new(
        position: Vec3,
        color: Vec4,
    ) -> Self {
        Self {
            position: RenderVec3::new(position.x, position.y, position.z).to_array(),
            color: RenderVec4::new(color.x, color.y, color.z, color.w).to_array(),
        }
    }
}
