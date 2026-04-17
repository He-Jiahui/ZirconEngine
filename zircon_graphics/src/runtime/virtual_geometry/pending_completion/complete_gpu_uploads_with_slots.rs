use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn complete_gpu_uploads_with_slots(
        &mut self,
        assignments: impl IntoIterator<Item = (u32, u32)>,
        evictable_pages: &[u32],
    ) {
        if self.page_budget == 0 {
            return;
        }

        let assignments = assignments
            .into_iter()
            .filter(|(page_id, _)| self.pending_pages.contains(page_id))
            .take(self.page_budget)
            .collect::<Vec<_>>();

        for (page_id, slot) in assignments {
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

                if !self.evict_one(evictable_pages.iter().copied()) {
                    self.evictable_pages
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident_in_slot(page_id, slot);
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
