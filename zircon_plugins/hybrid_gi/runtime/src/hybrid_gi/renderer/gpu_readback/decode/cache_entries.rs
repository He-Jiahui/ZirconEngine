use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::hybrid_gi::renderer::gpu_readback) fn cache_entries(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<(u32, u32)>, GraphicsError> {
    let cache_words = read_buffer_u32s(device, buffer, word_count)?;
    Ok(cache_words
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect())
}
