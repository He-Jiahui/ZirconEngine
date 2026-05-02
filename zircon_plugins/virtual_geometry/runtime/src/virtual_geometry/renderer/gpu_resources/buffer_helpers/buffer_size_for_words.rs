const U32_SIZE: u64 = std::mem::size_of::<u32>() as u64;

pub(in crate::virtual_geometry::renderer::gpu_resources) fn buffer_size_for_words(
    word_count: usize,
) -> u64 {
    (word_count.max(1) as u64) * U32_SIZE
}
