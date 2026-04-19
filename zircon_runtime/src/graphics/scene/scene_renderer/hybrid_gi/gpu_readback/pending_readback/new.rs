use super::HybridGiGpuPendingReadback;

impl HybridGiGpuPendingReadback {
    pub(in crate::graphics::scene::scene_renderer::hybrid_gi) fn new(
        cache_word_count: usize,
        cache_buffer: wgpu::Buffer,
        completed_probe_word_count: usize,
        completed_probe_buffer: wgpu::Buffer,
        completed_trace_word_count: usize,
        completed_trace_buffer: wgpu::Buffer,
        irradiance_word_count: usize,
        irradiance_buffer: wgpu::Buffer,
        trace_lighting_word_count: usize,
        trace_lighting_buffer: wgpu::Buffer,
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
            trace_lighting_word_count,
            trace_lighting_buffer,
        }
    }
}
