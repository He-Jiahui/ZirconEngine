use super::declarations::VirtualGeometryRuntimeSnapshot;
use super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn snapshot(&self) -> VirtualGeometryRuntimeSnapshot {
        VirtualGeometryRuntimeSnapshot {
            page_table_entry_count: self.resident_slots.len(),
            resident_page_count: self.resident_slots.len(),
            pending_request_count: self.pending_requests.len(),
        }
    }

    pub(crate) fn resident_slot_owners(&self) -> Vec<(u32, u32)> {
        self.resident_slots
            .iter()
            .map(|(&page_id, &slot)| (slot, page_id))
            .collect()
    }

    pub(crate) fn pending_page_ids(&self) -> Vec<u32> {
        self.pending_pages.iter().copied().collect()
    }
}
