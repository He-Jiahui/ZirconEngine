use std::collections::BTreeSet;

pub(crate) fn normalized_page_table_entries(page_table_entries: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut seen_page_ids = BTreeSet::new();
    let mut seen_slots = BTreeSet::new();
    let mut normalized_entries = Vec::new();

    for &(page_id, slot) in page_table_entries.iter().rev() {
        if seen_page_ids.contains(&page_id) || seen_slots.contains(&slot) {
            continue;
        }
        seen_page_ids.insert(page_id);
        seen_slots.insert(slot);
        normalized_entries.push((page_id, slot));
    }

    normalized_entries.sort_by_key(|(_page_id, slot)| *slot);
    normalized_entries
}
