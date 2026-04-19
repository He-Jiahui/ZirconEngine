use crate::{backend::read_buffer_u32s, types::GraphicsError};

pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_readback) fn completed_page_assignments(
    device: &wgpu::Device,
    completed_buffer: &wgpu::Buffer,
    completed_word_count: usize,
) -> Result<(Vec<(u32, u32)>, Vec<u32>, Vec<(u32, u32)>), GraphicsError> {
    let completed_words = read_buffer_u32s(device, completed_buffer, completed_word_count)?;
    let completed_count = completed_words.first().copied().unwrap_or_default() as usize;
    let completed_triplets = completed_words
        .into_iter()
        .skip(1)
        .take(completed_count.saturating_mul(3))
        .collect::<Vec<_>>();
    let completed_page_assignments: Vec<(u32, u32)> = completed_triplets
        .chunks_exact(3)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect();
    let completed_page_ids = completed_page_assignments
        .iter()
        .map(|(page_id, _)| *page_id)
        .collect();
    let completed_page_replacements = completed_triplets
        .chunks_exact(3)
        .filter_map(|chunk| (chunk[2] != u32::MAX).then_some((chunk[0], chunk[2])))
        .collect();

    Ok((
        completed_page_assignments,
        completed_page_ids,
        completed_page_replacements,
    ))
}
