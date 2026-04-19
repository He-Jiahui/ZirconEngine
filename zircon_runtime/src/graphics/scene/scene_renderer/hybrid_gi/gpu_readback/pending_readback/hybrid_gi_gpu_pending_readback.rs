pub(crate) struct HybridGiGpuPendingReadback {
    pub(super) cache_word_count: usize,
    pub(super) cache_buffer: wgpu::Buffer,
    pub(super) completed_probe_word_count: usize,
    pub(super) completed_probe_buffer: wgpu::Buffer,
    pub(super) completed_trace_word_count: usize,
    pub(super) completed_trace_buffer: wgpu::Buffer,
    pub(super) irradiance_word_count: usize,
    pub(super) irradiance_buffer: wgpu::Buffer,
    pub(super) trace_lighting_word_count: usize,
    pub(super) trace_lighting_buffer: wgpu::Buffer,
}
