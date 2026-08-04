use crate::core::math::UVec2;
use zr_rhi::RenderNativeSurfaceTarget;

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
