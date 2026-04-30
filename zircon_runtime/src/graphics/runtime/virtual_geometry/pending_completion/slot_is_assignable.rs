use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::pending_completion) fn slot_is_assignable(
        &self,
        slot: u32,
        evictable_pages: &[u32],
        target_page_id: u32,
    ) -> bool {
        match self.page_in_slot(slot) {
            Some(resident_page_id) => {
                resident_page_id == target_page_id || evictable_pages.contains(&resident_page_id)
            }
            None => self.has_free_slot(slot) || slot >= self.next_slot(),
        }
    }
}
