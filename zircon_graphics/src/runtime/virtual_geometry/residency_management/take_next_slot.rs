use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry::residency_management) fn take_next_slot(
        &mut self,
    ) -> u32 {
        let slot = self.next_slot;
        self.next_slot = self.next_slot.saturating_add(1);
        slot
    }
}
