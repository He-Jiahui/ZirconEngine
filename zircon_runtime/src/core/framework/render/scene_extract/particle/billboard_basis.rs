use crate::core::math::Vec3;

/// 保存上一帧相机的公告板基向量，使相机旋转时粒子速度仍能对齐历史几何。
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
