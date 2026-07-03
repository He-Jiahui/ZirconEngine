use crate::core::math::{Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVirtualGeometryPagePayloadVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec4,
}

impl Default for RenderVirtualGeometryPagePayloadVertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::new(0.0, 1.0, 0.0),
            tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderVirtualGeometryPagePayload {
    pub page_id: u32,
    pub vertices: Vec<RenderVirtualGeometryPagePayloadVertex>,
}

impl RenderVirtualGeometryPagePayload {
    pub fn new(page_id: u32, vertices: Vec<RenderVirtualGeometryPagePayloadVertex>) -> Self {
        Self { page_id, vertices }
    }
}
