use std::collections::BTreeSet;

use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn apply_gpu_page_table_entries(&mut self, page_table_entries: &[(u32, u32)]) {
        let resident_page_ids = self.resident_slots.keys().copied().collect::<Vec<_>>();
        let gpu_resident_pages = page_table_entries
            .iter()
            .map(|(page_id, _)| *page_id)
            .collect::<BTreeSet<_>>();

        for page_id in resident_page_ids {
            if !gpu_resident_pages.contains(&page_id) {
                self.evict_page(page_id);
            }
        }

        for (page_id, slot) in page_table_entries {
            self.promote_to_resident_in_slot(*page_id, *slot);
        }

        self.evictable_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
        self.current_hot_resident_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
    }
}
