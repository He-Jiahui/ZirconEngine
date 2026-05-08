use crate::core::math::Vec3;

pub(crate) struct GpuMeshResource {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    #[allow(dead_code)]
    pub(crate) indirect_order_signature: u64,
    pub(crate) wire_segments: Vec<[Vec3; 2]>,
    pub(crate) bounds_min: Vec3,
    pub(crate) bounds_max: Vec3,
}
