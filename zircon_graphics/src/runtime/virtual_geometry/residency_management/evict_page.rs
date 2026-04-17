use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry) fn evict_page(&mut self, page_id: u32) -> Option<u32> {
        let slot = self.resident_slots.remove(&page_id)?;
        self.free_slots.insert(slot);
        self.evictable_pages
            .retain(|candidate| *candidate != page_id);
        Some(slot)
    }
}
