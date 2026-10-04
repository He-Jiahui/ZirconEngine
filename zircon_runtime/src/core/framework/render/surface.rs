use crate::core::math::UVec2;
use zr_rhi::RenderNativeSurfaceTarget;

/// 宿主提供的原生展示目标与当前像素尺寸；仅用于绑定视口表面，
/// 不携带后端交换链或帧内图像的所有权。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderViewportSurfaceDescriptor {
    pub size: UVec2,
    pub target: RenderNativeSurfaceTarget,
}

impl RenderViewportSurfaceDescriptor {
    pub const fn new(size: UVec2, target: RenderNativeSurfaceTarget) -> Self {
        Self { size, target }
    }
}
