use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry::pending_completion) fn slot_is_assignable(
        &self,
        slot: u32,
        evictable_pages: &[u32],
        target_page_id: u32,
    ) -> bool {
        match self
            .resident_slots
            .iter()
            .find_map(|(&resident_page_id, &resident_slot)| {
                (resident_slot == slot).then_some(resident_page_id)
            }) {
            Some(resident_page_id) => {
                resident_page_id == target_page_id
                    || evictable_pages.contains(&resident_page_id)
                    || self.evictable_pages.contains(&resident_page_id)
            }
            None => self.free_slots.contains(&slot) || slot >= self.next_slot,
        }
    }
}
