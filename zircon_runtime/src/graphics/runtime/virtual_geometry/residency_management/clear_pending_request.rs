use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::residency_management) fn clear_pending_request(
        &mut self,
        page_id: u32,
    ) {
        self.remove_pending_page(page_id);
        self.retain_pending_page_requests(|request| request.page_id() != page_id);
    }
}
