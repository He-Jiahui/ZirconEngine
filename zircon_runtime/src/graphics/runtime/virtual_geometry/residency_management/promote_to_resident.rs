use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn promote_to_resident(
        &mut self,
        page_id: u32,
    ) {
        if self.has_resident_page(page_id) {
            return;
        }

        self.clear_pending_request(page_id);
        let slot = self
            .take_free_slot()
            .unwrap_or_else(|| self.take_next_slot());
        self.insert_resident_page_slot(page_id, slot);
    }
}
