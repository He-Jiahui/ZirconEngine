use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry) fn page_in_slot(&self, slot: u32) -> Option<u32> {
        self.resident_page_slots()
            .find_map(|(page_id, resident_slot)| (resident_slot == slot).then_some(page_id))
    }
}
