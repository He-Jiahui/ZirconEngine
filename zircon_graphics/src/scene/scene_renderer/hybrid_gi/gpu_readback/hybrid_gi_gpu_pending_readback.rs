pub(crate) struct HybridGiGpuPendingReadback {
    pub(super) cache_word_count: usize,
    pub(super) cache_buffer: wgpu::Buffer,
    pub(super) completed_probe_word_count: usize,
    pub(super) completed_probe_buffer: wgpu::Buffer,
    pub(super) completed_trace_word_count: usize,
    pub(super) completed_trace_buffer: wgpu::Buffer,
    pub(super) irradiance_word_count: usize,
    pub(super) irradiance_buffer: wgpu::Buffer,
}

impl HybridGiGpuPendingReadback {
    pub(crate) fn new(
        cache_word_count: usize,
        cache_buffer: wgpu::Buffer,
        completed_probe_word_count: usize,
        completed_probe_buffer: wgpu::Buffer,
        completed_trace_word_count: usize,
        completed_trace_buffer: wgpu::Buffer,
        irradiance_word_count: usize,
        irradiance_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            cache_word_count,
            cache_buffer,
            completed_probe_word_count,
            completed_probe_buffer,
            completed_trace_word_count,
            completed_trace_buffer,
            irradiance_word_count,
            irradiance_buffer,
        }
    }
}
