pub(crate) struct VirtualGeometryIndirectArgsGpuResources {
    pub(in crate::scene::scene_renderer::mesh) bind_group_layout: wgpu::BindGroupLayout,
    pub(in crate::scene::scene_renderer::mesh) pipeline: wgpu::ComputePipeline,
}
