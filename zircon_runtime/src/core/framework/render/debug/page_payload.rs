use crate::core::math::{Vec3, Vec4};

/// 页级载荷中的位置、法线与切线记录；驻留上传路径将其转换为 GPU 顶点，该 DTO 不持有缓冲所有权。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVirtualGeometryPagePayloadVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec4,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPagePayloadClusterRange {
    pub cluster_id: u32,
    pub vertex_start: u32,
    pub vertex_count: u32,
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
    pub cluster_ranges: Vec<RenderVirtualGeometryPagePayloadClusterRange>,
}

impl RenderVirtualGeometryPagePayload {
    pub fn new(page_id: u32, vertices: Vec<RenderVirtualGeometryPagePayloadVertex>) -> Self {
        Self {
            page_id,
            vertices,
            cluster_ranges: Vec::new(),
        }
    }

    pub fn with_cluster_ranges(
        mut self,
        cluster_ranges: Vec<RenderVirtualGeometryPagePayloadClusterRange>,
    ) -> Self {
        self.cluster_ranges = cluster_ranges;
        self
    }
}
