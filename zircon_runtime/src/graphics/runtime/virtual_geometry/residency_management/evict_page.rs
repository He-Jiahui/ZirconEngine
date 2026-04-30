use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn evict_page(
        &mut self,
        page_id: u32,
    ) -> Option<u32> {
        let slot = self.remove_resident_page_slot(page_id)?;
        self.insert_free_slot(slot);
        self.remove_hot_resident_page(page_id);
        self.remove_evictable_page(page_id);
        Some(slot)
    }
}
