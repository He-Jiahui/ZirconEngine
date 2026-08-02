use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::hybrid_gi::renderer::gpu_readback) fn completed_probe_ids(
    bytes: &[u8],
    word_count: usize,
) -> Result<Vec<u32>, GraphicsError> {
    let completed_probe_words = read_buffer_u32s(bytes, word_count)?;
    let completed_probe_count = completed_probe_words.first().copied().unwrap_or_default() as usize;
    Ok(completed_probe_words
        .into_iter()
        .skip(1)
        .take(completed_probe_count)
        .collect())
}
