use std::collections::BTreeMap;

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

        let replacement_by_page_id = replacements.into_iter().collect::<BTreeMap<_, _>>();
        let assignments = assignments
            .into_iter()
            .filter(|(page_id, _)| self.pending_pages.contains(page_id))
            .take(self.page_budget)
            .collect::<Vec<_>>();

        for (page_id, slot) in assignments {
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
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
