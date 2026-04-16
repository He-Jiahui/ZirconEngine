use zircon_math::Vec3;

pub(crate) struct GpuMeshResource {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
    pub(crate) wire_segments: Vec<[Vec3; 2]>,
    pub(crate) bounds_min: Vec3,
    pub(crate) bounds_max: Vec3,
}
