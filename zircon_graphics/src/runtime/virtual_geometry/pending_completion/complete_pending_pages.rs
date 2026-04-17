use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry::pending_completion) fn complete_pending_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
        evictable_pages: &[u32],
    ) {
        if self.page_budget == 0 {
            return;
        }

        let requested_pages = page_ids
            .into_iter()
            .filter(|page_id| self.pending_pages.contains(page_id))
            .take(self.page_budget)
            .collect::<Vec<_>>();

        for page_id in requested_pages {
            while self.resident_slots.len() >= self.page_budget {
                if !self.evict_one(evictable_pages.iter().copied()) {
                    self.evictable_pages
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident(page_id);
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
