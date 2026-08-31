use std::collections::HashSet;

use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::virtual_geometry::renderer::gpu_readback) fn page_table_entries(
    page_table_bytes: &[u8],
    page_table_word_count: usize,
    resident_entry_count: usize,
    resident_slots: Vec<u32>,
    completed_page_assignments: &[(u32, u32)],
) -> Result<Vec<(u32, u32)>, GraphicsError> {
    let page_table_words = read_buffer_u32s(page_table_bytes, page_table_word_count)?;
    Ok(project_page_table_entries(
        &page_table_words,
        resident_entry_count,
        &resident_slots,
        completed_page_assignments,
    ))
}

fn project_page_table_entries(
    page_table_words: &[u32],
    resident_entry_count: usize,
    resident_slots: &[u32],
    completed_page_assignments: &[(u32, u32)],
) -> Vec<(u32, u32)> {
    let mut resident_slot_index = HashSet::with_capacity(resident_slots.len());
    resident_slot_index.extend(resident_slots.iter().copied());
    let appended_entry_count = completed_page_assignments
        .iter()
        .filter(|(_, slot)| !resident_slot_index.contains(slot))
        .count();
    let page_table_entry_count = resident_entry_count
        .saturating_add(appended_entry_count)
        .min(page_table_words.len() / 2);
    let mut entries = Vec::with_capacity(page_table_entry_count);
    for chunk in page_table_words
        .chunks_exact(2)
        .take(page_table_entry_count)
    {
        entries.push((chunk[0], chunk[1]));
    }
    entries
}

#[cfg(test)]
mod performance_tests;
