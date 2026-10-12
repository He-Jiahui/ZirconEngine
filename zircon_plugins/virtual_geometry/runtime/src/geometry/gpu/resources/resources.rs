pub(crate) struct VirtualGeometryGpuResources {
    pub(in crate::virtual_geometry::renderer::gpu_resources) bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::virtual_geometry::renderer::gpu_resources) pipeline: wgpu::ComputePipeline,
    pub(in crate::virtual_geometry::renderer::gpu_resources) params_buffer: wgpu::Buffer,
    pub(super) node_and_cluster_cull_instance_work_item_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) node_and_cluster_cull_instance_work_item_pipeline: wgpu::ComputePipeline,
}
