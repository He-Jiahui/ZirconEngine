use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry) fn promote_to_resident(&mut self, page_id: u32) {
        if self.resident_slots.contains_key(&page_id) {
            return;
        }

        self.clear_pending_request(page_id);
        let slot = self
            .take_free_slot()
            .unwrap_or_else(|| self.take_next_slot());
        self.resident_slots.insert(page_id, slot);
    }
}
