use std::collections::{BTreeMap, BTreeSet};

use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn complete_gpu_uploads_with_replacements(
        &mut self,
        assignments: impl IntoIterator<Item = (u32, u32)>,
        replacements: impl IntoIterator<Item = (u32, u32)>,
        evictable_pages: &[u32],
    ) {
        let page_budget = self.page_budget();
        if page_budget == 0 {
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
            if !self.has_pending_page(page_id) || !seen_page_ids.insert(page_id) {
                continue;
            }
            assignments_by_first_unique_page.push((page_id, slot));
        }
        let frontier_hot_pages = self.frontier_hot_resident_pages();
        let displaced_resident_pages = assignments_by_first_unique_page
            .iter()
            .filter_map(|(page_id, slot)| {
                self.resident_page_slots()
                    .find_map(|(resident_page_id, resident_slot)| {
                        (resident_slot == *slot && resident_page_id != *page_id)
                            .then_some(resident_page_id)
                    })
            })
            .collect::<BTreeSet<_>>();
        let surviving_frontier_hot_pages = frontier_hot_pages
            .iter()
            .copied()
            .filter(|page_id| !displaced_resident_pages.contains(page_id))
            .collect::<BTreeSet<_>>();

        for (page_id, slot) in assignments_by_first_unique_page {
            let confirmed_replaced_page_id = replacement_by_page_id
                .get(&page_id)
                .copied()
                .filter(|replaced_page_id| self.resident_slot(*replaced_page_id) == Some(slot));
            let inherits_hot_frontier = confirmed_replaced_page_id
                .map(|replaced_page_id| frontier_hot_pages.contains(&replaced_page_id))
                .unwrap_or(false)
                || self.page_or_lineage_is_hot_in(page_id, &surviving_frontier_hot_pages);
            if let Some(replaced_page_id) = confirmed_replaced_page_id {
                self.evict_page(replaced_page_id);
            }

            if !self.slot_is_assignable(slot, evictable_pages, page_id) {
                continue;
            }

            while self.resident_page_count() >= page_budget {
                if self.page_in_slot(slot).is_some() {
                    break;
                }

                if !self
                    .evict_one(self.ordered_evictable_pages_for_target(page_id, evictable_pages))
                {
                    self.retain_resident_evictable_pages();
                    return;
                }
            }

            self.promote_to_resident_in_slot(page_id, slot);
            if inherits_hot_frontier {
                self.insert_current_hot_resident_page(page_id);
            }
        }

        self.retain_resident_evictable_pages();
    }
}
