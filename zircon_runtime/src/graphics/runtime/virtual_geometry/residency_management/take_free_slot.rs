use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::residency_management) fn take_free_slot(
        &mut self,
    ) -> Option<u32> {
        let slot = self.free_slots.iter().next().copied()?;
        self.free_slots.remove(&slot);
        Some(slot)
    }
}
