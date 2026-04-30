pub(in crate::graphics::scene::scene_renderer) struct VirtualGeometryIndirectArgsGpuResources {
    pub(in crate::graphics::scene::scene_renderer::mesh) bind_group_layout: wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) pipeline: wgpu::ComputePipeline,
}
