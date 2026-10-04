use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::hybrid_gi::renderer::gpu_readback) fn cache_entries(
    bytes: &[u8],
    word_count: usize,
) -> Result<Vec<(u32, u32)>, GraphicsError> {
    let cache_words = read_buffer_u32s(bytes, word_count)?;
    // 此缓冲按 (probe_id, slot) 连续成对初始化，不采用其他读回缓冲的首字计数格式。
    Ok(cache_words
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect())
}
