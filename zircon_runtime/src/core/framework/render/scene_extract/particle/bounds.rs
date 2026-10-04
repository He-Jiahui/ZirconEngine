use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

/// 粒子实体的聚合空间边界，随帧提取供可见性与 GPU 粒子路径消费。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticleBoundsSnapshot {
    pub entity: EntityId,
    pub center: Vec3,
    pub radius: Real,
}
