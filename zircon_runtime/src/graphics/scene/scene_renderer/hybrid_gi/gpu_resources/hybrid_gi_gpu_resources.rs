pub(in crate::graphics::scene::scene_renderer) struct HybridGiGpuResources {
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::ComputePipeline,
    pub(super) params_buffer: wgpu::Buffer,
}
