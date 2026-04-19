use super::{
    VirtualGeometryPageRequest, VirtualGeometryPageResidencyState, VirtualGeometryRuntimeState,
};

impl VirtualGeometryRuntimeState {
    pub(crate) fn page_slot(&self, page_id: u32) -> Option<u32> {
        self.resident_slots.get(&page_id).copied()
    }

    pub(crate) fn page_residency(&self, page_id: u32) -> Option<VirtualGeometryPageResidencyState> {
        if self.resident_slots.contains_key(&page_id) {
            return Some(VirtualGeometryPageResidencyState::Resident);
        }
        if self.pending_pages.contains(&page_id) {
            return Some(VirtualGeometryPageResidencyState::PendingUpload);
        }
        None
    }

    pub(crate) fn pending_requests(&self) -> Vec<VirtualGeometryPageRequest> {
        self.pending_requests.clone()
    }

    pub(crate) fn evictable_pages(&self) -> Vec<u32> {
        self.evictable_pages.clone()
    }

    pub(crate) fn apply_evictions(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        for page_id in page_ids {
            self.evict_page(page_id);
        }
        self.evictable_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
    }

    pub(crate) fn fulfill_requests(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        for page_id in page_ids {
            if !self.pending_pages.remove(&page_id) {
                continue;
            }

            self.pending_requests
                .retain(|request| request.page_id != page_id);
            self.promote_to_resident(page_id);
        }
    }
}
