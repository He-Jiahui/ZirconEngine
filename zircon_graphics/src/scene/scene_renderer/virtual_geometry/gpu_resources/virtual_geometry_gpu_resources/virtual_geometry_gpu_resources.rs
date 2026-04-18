pub(crate) struct VirtualGeometryGpuResources {
    pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) pipeline:
        wgpu::ComputePipeline,
    pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) params_buffer:
        wgpu::Buffer,
}
