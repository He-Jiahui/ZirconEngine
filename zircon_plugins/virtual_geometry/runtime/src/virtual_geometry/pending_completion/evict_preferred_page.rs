use super::super::VirtualGeometryRuntimeState;

#[cfg(test)]
#[path = "evict_preferred_page/allocation_tests.rs"]
mod allocation_tests;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry) fn preferred_resident_evictable_page_for_target(
        &self,
        target_page_id: u32,
        evictable_pages: &[u32],
    ) -> Option<u32> {
        let resident_evictable_pages = evictable_pages
            .iter()
            .copied()
            .filter(|page_id| self.has_resident_page(*page_id))
            .collect::<Vec<_>>();
        self.preferred_evictable_page_for_target(target_page_id, &resident_evictable_pages)
    }

    pub(in crate::virtual_geometry) fn evict_preferred_page_for_target(
        &mut self,
        target_page_id: u32,
        evictable_pages: &[u32],
    ) -> bool {
        self.preferred_resident_evictable_page_for_target(target_page_id, evictable_pages)
            .and_then(|page_id| self.evict_page(page_id))
            .is_some()
    }
}
