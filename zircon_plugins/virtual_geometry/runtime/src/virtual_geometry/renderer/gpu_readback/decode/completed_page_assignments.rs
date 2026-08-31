use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::virtual_geometry::renderer::gpu_readback) fn completed_page_assignments(
    completed_bytes: &[u8],
    completed_word_count: usize,
) -> Result<(Vec<(u32, u32)>, Vec<u32>, Vec<(u32, u32)>), GraphicsError> {
    let completed_words = read_buffer_u32s(completed_bytes, completed_word_count)?;
    Ok(project_completed_assignments(&completed_words))
}

fn project_completed_assignments(
    completed_words: &[u32],
) -> (Vec<(u32, u32)>, Vec<u32>, Vec<(u32, u32)>) {
    let completed_count = completed_words.first().copied().unwrap_or_default() as usize;
    let completed_triplets = completed_words.get(1..).unwrap_or_default();
    let available_count = completed_count.min(completed_triplets.len() / 3);
    let mut completed_page_assignments = Vec::with_capacity(available_count);
    let mut completed_page_ids = Vec::with_capacity(available_count);
    let mut completed_page_replacements = Vec::with_capacity(available_count);

    for chunk in completed_triplets.chunks_exact(3).take(available_count) {
        let page_id = chunk[0];
        completed_page_assignments.push((page_id, chunk[1]));
        completed_page_ids.push(page_id);
        if chunk[2] != u32::MAX {
            completed_page_replacements.push((page_id, chunk[2]));
        }
    }

    (
        completed_page_assignments,
        completed_page_ids,
        completed_page_replacements,
    )
}

#[cfg(test)]
mod performance_tests;
