use zircon_runtime::graphics::VisibilityVirtualGeometryPageUploadPlan;

use super::{VirtualGeometryPageRequest, VirtualGeometryRuntimeState};

impl VirtualGeometryRuntimeState {
    pub(crate) fn ingest_plan(
        &mut self,
        generation: u64,
        plan: &VisibilityVirtualGeometryPageUploadPlan,
    ) {
        self.clear_current_requested_page_order();
        for (order, page_id) in plan.requested_pages.iter().copied().enumerate() {
            self.ensure_current_requested_page_order(page_id, order);
        }

        for &page_id in &plan.resident_pages {
            self.promote_to_resident(page_id);
        }

        for &page_id in &plan.dirty_requested_pages {
            if self.has_resident_page(page_id) || self.has_pending_page(page_id) {
                continue;
            }

            let size_bytes = self.page_size_bytes(page_id);
            self.insert_pending_page(page_id);
            self.push_pending_page_request(VirtualGeometryPageRequest::new(
                page_id, size_bytes, generation,
            ));
        }

        let evictable_pages = plan
            .evictable_pages
            .iter()
            .copied()
            .filter(|page_id| self.has_resident_page(*page_id))
            .collect();
        self.replace_evictable_pages(evictable_pages);
    }

    pub(crate) fn ingest_page_requests(
        &mut self,
        generation: u64,
        page_ids: impl IntoIterator<Item = u32>,
    ) {
        let mut next_order = self.current_requested_page_order_len();
        for page_id in page_ids {
            if !self.has_page_size(page_id) {
                continue;
            }

            if self.ensure_current_requested_page_order(page_id, next_order) {
                next_order += 1;
            }

            if self.has_resident_page(page_id) || self.has_pending_page(page_id) {
                continue;
            }

            let size_bytes = self.page_size_bytes(page_id);
            self.insert_pending_page(page_id);
            self.push_pending_page_request(VirtualGeometryPageRequest::new(
                page_id, size_bytes, generation,
            ));
        }
    }
}
