use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry::residency_management) fn page_in_slot(
        &self,
        slot: u32,
    ) -> Option<u32> {
        self.resident_slots
            .iter()
            .find_map(|(&page_id, &resident_slot)| (resident_slot == slot).then_some(page_id))
    }
}
