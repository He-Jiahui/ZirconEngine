use crate::{backend::read_buffer_u32s, types::GraphicsError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(crate) cache_entries: Vec<(u32, u32)>,
    pub(crate) completed_probe_ids: Vec<u32>,
    pub(crate) completed_trace_region_ids: Vec<u32>,
}

pub(crate) struct HybridGiGpuPendingReadback {
    cache_word_count: usize,
    cache_buffer: wgpu::Buffer,
    completed_probe_word_count: usize,
    completed_probe_buffer: wgpu::Buffer,
    completed_trace_word_count: usize,
    completed_trace_buffer: wgpu::Buffer,
}

impl HybridGiGpuPendingReadback {
    pub(crate) fn new(
        cache_word_count: usize,
        cache_buffer: wgpu::Buffer,
        completed_probe_word_count: usize,
        completed_probe_buffer: wgpu::Buffer,
        completed_trace_word_count: usize,
        completed_trace_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            cache_word_count,
            cache_buffer,
            completed_probe_word_count,
            completed_probe_buffer,
            completed_trace_word_count,
            completed_trace_buffer,
        }
    }

    pub(crate) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<HybridGiGpuReadback, GraphicsError> {
        let cache_words = read_buffer_u32s(device, &self.cache_buffer, self.cache_word_count)?;
        let cache_entries = cache_words
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();

        let completed_probe_words = read_buffer_u32s(
            device,
            &self.completed_probe_buffer,
            self.completed_probe_word_count,
        )?;
        let completed_probe_count =
            completed_probe_words.first().copied().unwrap_or_default() as usize;
        let completed_probe_ids = completed_probe_words
            .into_iter()
            .skip(1)
            .take(completed_probe_count)
            .collect();

        let completed_trace_words = read_buffer_u32s(
            device,
            &self.completed_trace_buffer,
            self.completed_trace_word_count,
        )?;
        let completed_trace_count =
            completed_trace_words.first().copied().unwrap_or_default() as usize;
        let completed_trace_region_ids = completed_trace_words
            .into_iter()
            .skip(1)
            .take(completed_trace_count)
            .collect();

        Ok(HybridGiGpuReadback {
            cache_entries,
            completed_probe_ids,
            completed_trace_region_ids,
        })
    }
}
