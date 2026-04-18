use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry) fn promote_to_resident_in_slot(
        &mut self,
        page_id: u32,
        slot: u32,
    ) {
        self.clear_pending_request(page_id);
        if let Some(previous_slot) = self.resident_slots.remove(&page_id) {
            if previous_slot != slot {
                self.free_slots.insert(previous_slot);
            }
        }
        if let Some(conflicting_page) = self.page_in_slot(slot) {
            if conflicting_page != page_id {
                self.evict_page(conflicting_page);
            }
        }
        self.reserve_slot(slot);
        self.resident_slots.insert(page_id, slot);
    }
}
