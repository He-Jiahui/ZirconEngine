use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry::residency_management) fn clear_pending_request(
        &mut self,
        page_id: u32,
    ) {
        self.pending_pages.remove(&page_id);
        self.pending_requests
            .retain(|request| request.page_id != page_id);
    }
}
