use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::pending_completion) fn complete_pending_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
        evictable_pages: &[u32],
    ) {
        if self.page_budget == 0 {
            return;
        }

        let mut requested_pages = Vec::new();
        let mut seen_page_ids = std::collections::BTreeSet::new();
        for page_id in page_ids {
            if !self.pending_pages.contains(&page_id) || !seen_page_ids.insert(page_id) {
                continue;
            }
            requested_pages.push(page_id);
            if requested_pages.len() >= self.page_budget {
                break;
            }
        }

        for page_id in requested_pages {
            let inherits_hot_frontier = self.page_or_lineage_is_hot(page_id);
            while self.resident_slots.len() >= self.page_budget {
                if !self
                    .evict_one(self.ordered_evictable_pages_for_target(page_id, evictable_pages))
                {
                    self.evictable_pages
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident(page_id);
            if inherits_hot_frontier {
                self.current_hot_resident_pages.insert(page_id);
            }
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
