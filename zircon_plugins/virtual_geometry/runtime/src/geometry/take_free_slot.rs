use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry::residency_management) fn take_free_slot(
        &mut self,
    ) -> Option<u32> {
        let slot = self.first_free_slot()?;
        self.remove_free_slot(slot);
        Some(slot)
    }
}
