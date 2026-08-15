pub(in crate::hybrid_gi::renderer) struct HybridGiGpuResources {
    pub(super) global_sdf: super::global_sdf::GlobalSdfGpuResources,
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::ComputePipeline,
    pub(super) params_buffer: wgpu::Buffer,
    pub(super) probe_trace_tile_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) probe_trace_tile_pipeline: wgpu::ComputePipeline,
    pub(super) probe_trace_tile_generation_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) probe_trace_tile_generation_pipeline: wgpu::ComputePipeline,
    pub(super) radiance_cache_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) radiance_cache_update_pipeline: wgpu::ComputePipeline,
    pub(super) radiance_cache_consume_pipeline: wgpu::ComputePipeline,
}
