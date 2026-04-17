use crate::{backend::read_buffer_u32s, types::GraphicsError};

pub(super) fn completed_trace_region_ids(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, GraphicsError> {
    let completed_trace_words = read_buffer_u32s(device, buffer, word_count)?;
    let completed_trace_count = completed_trace_words.first().copied().unwrap_or_default() as usize;
    Ok(completed_trace_words
        .into_iter()
        .skip(1)
        .take(completed_trace_count)
        .collect())
}
