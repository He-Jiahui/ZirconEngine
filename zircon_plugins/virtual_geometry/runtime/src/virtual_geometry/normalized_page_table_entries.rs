use std::collections::HashSet;

// 倒序保留页 ID 与槽位均唯一的最后一条，再按槽位排序，供回读完成和页表同步共用。
pub(crate) fn normalized_page_table_entries(page_table_entries: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut seen_page_ids = HashSet::with_capacity(page_table_entries.len());
    let mut seen_slots = HashSet::with_capacity(page_table_entries.len());
    let mut normalized_entries = Vec::with_capacity(page_table_entries.len());

    for &(page_id, slot) in page_table_entries.iter().rev() {
        if seen_page_ids.contains(&page_id) || seen_slots.contains(&slot) {
            continue;
        }
        seen_page_ids.insert(page_id);
        seen_slots.insert(slot);
        normalized_entries.push((page_id, slot));
    }

    normalized_entries.sort_unstable_by_key(|(_page_id, slot)| *slot);
    normalized_entries
}

#[cfg(test)]
#[path = "normalized_page_table_entries/tests/allocation_tests.rs"]
mod allocation_tests;
