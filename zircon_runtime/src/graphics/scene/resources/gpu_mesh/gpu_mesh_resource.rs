use crate::core::math::Vec3;

pub(crate) struct GpuMeshResource {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    pub(super) indirect_order_signature: u64,
    pub(crate) wire_segments: Vec<[Vec3; 2]>,
    pub(crate) bounds_min: Vec3,
    pub(crate) bounds_max: Vec3,
}

impl GpuMeshResource {
    pub(crate) const fn indirect_order_signature(&self) -> u64 {
        self.indirect_order_signature
    }
}
