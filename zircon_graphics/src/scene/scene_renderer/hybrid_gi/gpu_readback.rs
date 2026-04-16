use crate::{backend::read_buffer_u32s, types::GraphicsError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(crate) cache_entries: Vec<(u32, u32)>,
    pub(crate) completed_probe_ids: Vec<u32>,
    pub(crate) completed_trace_region_ids: Vec<u32>,
    pub(crate) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
}

pub(crate) struct HybridGiGpuPendingReadback {
    cache_word_count: usize,
    cache_buffer: wgpu::Buffer,
    completed_probe_word_count: usize,
    completed_probe_buffer: wgpu::Buffer,
    completed_trace_word_count: usize,
    completed_trace_buffer: wgpu::Buffer,
    irradiance_word_count: usize,
    irradiance_buffer: wgpu::Buffer,
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

        let irradiance_words =
            read_buffer_u32s(device, &self.irradiance_buffer, self.irradiance_word_count)?;
        let irradiance_count = irradiance_words.first().copied().unwrap_or_default() as usize;
        let probe_irradiance_rgb = irradiance_words
            .into_iter()
            .skip(1)
            .collect::<Vec<_>>()
            .chunks_exact(2)
            .take(irradiance_count)
            .map(|chunk| (chunk[0], unpack_rgb8(chunk[1])))
            .collect();

        Ok(HybridGiGpuReadback {
            cache_entries,
            completed_probe_ids,
            completed_trace_region_ids,
            probe_irradiance_rgb,
        })
    }
}

fn unpack_rgb8(packed: u32) -> [u8; 3] {
    [
        (packed & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        ((packed >> 16) & 0xff) as u8,
    ]
}
