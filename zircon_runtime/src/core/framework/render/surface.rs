use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderNativeSurfaceTarget {
    Win32 { hwnd: u64, hinstance: Option<u64> },
}

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
