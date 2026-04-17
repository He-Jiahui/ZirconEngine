use super::virtual_geometry_runtime_snapshot::VirtualGeometryRuntimeSnapshot;
use super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(crate) fn snapshot(&self) -> VirtualGeometryRuntimeSnapshot {
        VirtualGeometryRuntimeSnapshot {
            page_table_entry_count: self.resident_slots.len(),
            resident_page_count: self.resident_slots.len(),
            pending_request_count: self.pending_requests.len(),
        }
    }
}
