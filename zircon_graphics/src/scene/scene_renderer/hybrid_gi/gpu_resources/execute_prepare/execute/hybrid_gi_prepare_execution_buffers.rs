pub(super) struct HybridGiPrepareExecutionBuffers {
    pub(super) cache_readback: wgpu::Buffer,
    pub(super) resident_probe_buffer: wgpu::Buffer,
    pub(super) pending_probe_buffer: wgpu::Buffer,
    pub(super) trace_region_buffer: wgpu::Buffer,
    pub(super) completed_probe_buffer: wgpu::Buffer,
    pub(super) completed_trace_buffer: wgpu::Buffer,
    pub(super) completed_probe_readback: wgpu::Buffer,
    pub(super) completed_trace_readback: wgpu::Buffer,
    pub(super) irradiance_buffer: wgpu::Buffer,
    pub(super) irradiance_readback: wgpu::Buffer,
}
