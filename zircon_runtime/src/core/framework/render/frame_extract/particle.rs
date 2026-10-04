use crate::core::framework::scene::EntityId;

use super::super::{
    RenderParticleBoundsSnapshot, RenderParticlePreviousSpriteSnapshot,
    RenderParticleSpriteSnapshot,
};
use super::RenderParticleGpuFrameExtract;

/// 粒子系统跨场景与渲染边界的提交包，包含当前实例、历史实例及 GPU 统计侧带。
/// 运动向量历史可由渲染提交上下文另行覆盖，不能假定此处总携带前帧数据。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ParticleExtract {
    pub emitters: Vec<EntityId>,
    pub sprites: Vec<RenderParticleSpriteSnapshot>,
    pub previous_sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
    pub bounds: Vec<RenderParticleBoundsSnapshot>,
    pub sort_camera_position: Option<crate::core::math::Vec3>,
    pub gpu_frame: Option<RenderParticleGpuFrameExtract>,
}
