use std::collections::BTreeSet;

use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::virtual_geometry::renderer::gpu_readback) fn page_table_entries(
    device: &wgpu::Device,
    page_table_buffer: &wgpu::Buffer,
    page_table_word_count: usize,
    resident_entry_count: usize,
    resident_slots: Vec<u32>,
    completed_page_assignments: &[(u32, u32)],
) -> Result<Vec<(u32, u32)>, GraphicsError> {
    let page_table_words = read_buffer_u32s(device, page_table_buffer, page_table_word_count)?;
    let resident_slots = resident_slots.into_iter().collect::<BTreeSet<_>>();
    let appended_entry_count = completed_page_assignments
        .iter()
        .filter(|(_, slot)| !resident_slots.contains(slot))
        .count();
    let page_table_entry_count = resident_entry_count + appended_entry_count;

    Ok(page_table_words
        .chunks_exact(2)
        .take(page_table_entry_count)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect())
}
