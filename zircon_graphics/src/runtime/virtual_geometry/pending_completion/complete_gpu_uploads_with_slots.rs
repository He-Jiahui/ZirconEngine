use std::collections::{BTreeMap, BTreeSet};

use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    #[cfg(test)]
    pub(crate) fn complete_gpu_uploads_with_slots(
        &mut self,
        assignments: impl IntoIterator<Item = (u32, u32)>,
        evictable_pages: &[u32],
    ) {
        self.complete_gpu_uploads_with_replacements(
            assignments,
            std::iter::empty::<(u32, u32)>(),
            evictable_pages,
        );
    }

    pub(crate) fn complete_gpu_uploads_with_replacements(
        &mut self,
        assignments: impl IntoIterator<Item = (u32, u32)>,
        replacements: impl IntoIterator<Item = (u32, u32)>,
        evictable_pages: &[u32],
    ) {
        if self.page_budget == 0 {
            return;
        }

        let mut replacement_by_page_id = BTreeMap::new();
        for (page_id, replaced_page_id) in replacements {
            replacement_by_page_id
                .entry(page_id)
                .or_insert(replaced_page_id);
        }
        let mut assignments_by_first_unique_page = Vec::new();
        let mut seen_page_ids = BTreeSet::new();
        for (page_id, slot) in assignments {
            if !self.pending_pages.contains(&page_id) || !seen_page_ids.insert(page_id) {
                continue;
            }
            assignments_by_first_unique_page.push((page_id, slot));
        }

        for (page_id, slot) in assignments_by_first_unique_page {
            let inherits_hot_frontier = replacement_by_page_id
                .get(&page_id)
                .map(|replaced_page_id| self.page_is_frontier_hot(*replaced_page_id))
                .unwrap_or(false)
                || self.page_or_lineage_is_hot(page_id);
            if let Some(replaced_page_id) = replacement_by_page_id.get(&page_id).copied() {
                if self.resident_slots.get(&replaced_page_id).copied() == Some(slot) {
                    self.evict_page(replaced_page_id);
                }
            }

            if !self.slot_is_assignable(slot, evictable_pages, page_id) {
                continue;
            }

            while self.resident_slots.len() >= self.page_budget {
                if self
                    .resident_slots
                    .iter()
                    .find_map(|(&resident_page_id, &resident_slot)| {
                        (resident_slot == slot).then_some(resident_page_id)
                    })
                    .is_some()
                {
                    break;
                }

                if !self
                    .evict_one(self.ordered_evictable_pages_for_target(page_id, evictable_pages))
                {
                    self.evictable_pages
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident_in_slot(page_id, slot);
            if inherits_hot_frontier {
                self.current_hot_resident_pages.insert(page_id);
            }
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
