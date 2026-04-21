use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn evict_page(
        &mut self,
        page_id: u32,
    ) -> Option<u32> {
        let slot = self.resident_slots.remove(&page_id)?;
        self.free_slots.insert(slot);
        self.current_hot_resident_pages.remove(&page_id);
        self.recent_hot_resident_pages.remove(&page_id);
        self.evictable_pages
            .retain(|candidate| *candidate != page_id);
        Some(slot)
    }
}
