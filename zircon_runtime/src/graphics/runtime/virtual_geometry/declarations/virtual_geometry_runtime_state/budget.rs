use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn page_budget(&self) -> usize {
        self.page_budget
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn set_page_budget(
        &mut self,
        page_budget: usize,
    ) {
        self.page_budget = page_budget;
    }
}
