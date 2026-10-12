use super::declarations::VirtualGeometryRuntimeSnapshot;
use super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn snapshot(&self) -> VirtualGeometryRuntimeSnapshot {
        VirtualGeometryRuntimeSnapshot::new(
            self.resident_page_count(),
            self.resident_page_count(),
            self.pending_request_count(),
            self.page_dependency_count(),
        )
    }

    pub(crate) fn resident_slot_owners(&self) -> Vec<(u32, u32)> {
        self.resident_page_slots()
            .map(|(page_id, slot)| (slot, page_id))
            .collect()
    }

    pub(crate) fn pending_page_ids(&self) -> Vec<u32> {
        self.pending_page_id_iter().collect()
    }
}
