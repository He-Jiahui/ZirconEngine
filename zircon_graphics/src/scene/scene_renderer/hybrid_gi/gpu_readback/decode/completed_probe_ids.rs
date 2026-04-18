use crate::{backend::read_buffer_u32s, types::GraphicsError};

pub(in crate::scene::scene_renderer::hybrid_gi::gpu_readback) fn completed_probe_ids(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, GraphicsError> {
    let completed_probe_words = read_buffer_u32s(device, buffer, word_count)?;
    let completed_probe_count = completed_probe_words.first().copied().unwrap_or_default() as usize;
    Ok(completed_probe_words
        .into_iter()
        .skip(1)
        .take(completed_probe_count)
        .collect())
}
