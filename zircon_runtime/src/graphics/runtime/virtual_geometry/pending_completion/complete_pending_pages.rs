use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::pending_completion) fn complete_pending_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
        evictable_pages: &[u32],
    ) {
        let page_budget = self.page_budget();
        if page_budget == 0 {
            return;
        }

        let mut requested_pages = Vec::new();
        let mut seen_page_ids = std::collections::BTreeSet::new();
        for page_id in page_ids {
            if !self.has_pending_page(page_id) || !seen_page_ids.insert(page_id) {
                continue;
            }
            requested_pages.push(page_id);
            if requested_pages.len() >= page_budget {
                break;
            }
        }

        for page_id in requested_pages {
            let inherits_hot_frontier = self.page_or_lineage_is_hot(page_id);
            while self.resident_page_count() >= page_budget {
                if !self
                    .evict_one(self.ordered_evictable_pages_for_target(page_id, evictable_pages))
                {
                    self.retain_resident_evictable_pages();
                    return;
                }
            }

            self.promote_to_resident(page_id);
            if inherits_hot_frontier {
                self.insert_current_hot_resident_page(page_id);
            }
        }

        self.retain_resident_evictable_pages();
    }
}
