pub(in crate::graphics::scene::scene_renderer) struct VirtualGeometryGpuResources {
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) params_buffer:
        wgpu::Buffer,
    pub(super) node_and_cluster_cull_instance_work_item_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) node_and_cluster_cull_instance_work_item_pipeline: wgpu::ComputePipeline,
}
