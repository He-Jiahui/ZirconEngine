use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::residency_management) fn take_next_slot(
        &mut self,
    ) -> u32 {
        self.allocate_next_slot()
    }
}
