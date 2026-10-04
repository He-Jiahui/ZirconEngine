use crate::core::math::{RenderVec3, Vec3};

use super::ParticleVelocityVertex;

impl ParticleVelocityVertex {
    /// 保留当前与上一帧的对应世界角点供速度 shader 求位移；此处不验证有限值，快照准入边界待 CR-SCENE-PARTICLE-0002 确认。
    pub(in crate::graphics::scene::scene_renderer::particle) fn new(
        current_position: Vec3,
        previous_position: Vec3,
    ) -> Self {
        Self {
            current_position: RenderVec3::new(
                current_position.x,
                current_position.y,
                current_position.z,
            )
            .to_array(),
            previous_position: RenderVec3::new(
                previous_position.x,
                previous_position.y,
                previous_position.z,
            )
            .to_array(),
        }
    }
}
