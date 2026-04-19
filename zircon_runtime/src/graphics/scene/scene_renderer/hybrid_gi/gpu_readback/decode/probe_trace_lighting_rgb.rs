use crate::{backend::read_buffer_u32s, types::GraphicsError};

pub(in crate::graphics::scene::scene_renderer::hybrid_gi::gpu_readback) fn probe_trace_lighting_rgb(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<(u32, [u8; 3])>, GraphicsError> {
    let trace_lighting_words = read_buffer_u32s(device, buffer, word_count)?;
    let trace_lighting_count = trace_lighting_words.first().copied().unwrap_or_default() as usize;

    Ok(trace_lighting_words
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>()
        .chunks_exact(2)
        .take(trace_lighting_count)
        .map(|chunk| (chunk[0], unpack_rgb8(chunk[1])))
        .collect())
}

fn unpack_rgb8(packed: u32) -> [u8; 3] {
    [
        (packed & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        ((packed >> 16) & 0xff) as u8,
    ]
}
