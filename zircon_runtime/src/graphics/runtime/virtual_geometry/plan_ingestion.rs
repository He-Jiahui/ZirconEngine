use crate::VisibilityVirtualGeometryPageUploadPlan;

use super::{VirtualGeometryPageRequest, VirtualGeometryRuntimeState};

impl VirtualGeometryRuntimeState {
    pub(crate) fn ingest_plan(
        &mut self,
        generation: u64,
        plan: &VisibilityVirtualGeometryPageUploadPlan,
    ) {
        self.current_requested_page_order.clear();
        for (order, page_id) in plan.requested_pages.iter().copied().enumerate() {
            self.current_requested_page_order
                .entry(page_id)
                .or_insert(order);
        }

        for &page_id in &plan.resident_pages {
            self.promote_to_resident(page_id);
        }

        for &page_id in &plan.dirty_requested_pages {
            if self.resident_slots.contains_key(&page_id) || self.pending_pages.contains(&page_id) {
                continue;
            }

            self.pending_pages.insert(page_id);
            self.pending_requests.push(VirtualGeometryPageRequest {
                page_id,
                size_bytes: self.page_sizes.get(&page_id).copied().unwrap_or_default(),
                generation,
            });
        }

        self.evictable_pages = plan
            .evictable_pages
            .iter()
            .copied()
            .filter(|page_id| self.resident_slots.contains_key(page_id))
            .collect();
    }

    pub(crate) fn ingest_page_requests(
        &mut self,
        generation: u64,
        page_ids: impl IntoIterator<Item = u32>,
    ) {
        let mut next_order = self.current_requested_page_order.len();
        for page_id in page_ids {
            if !self.page_sizes.contains_key(&page_id) {
                continue;
            }

            if let std::collections::btree_map::Entry::Vacant(entry) =
                self.current_requested_page_order.entry(page_id)
            {
                entry.insert(next_order);
                next_order += 1;
            }

            if self.resident_slots.contains_key(&page_id) || self.pending_pages.contains(&page_id) {
                continue;
            }

            self.pending_pages.insert(page_id);
            self.pending_requests.push(VirtualGeometryPageRequest {
                page_id,
                size_bytes: self.page_sizes.get(&page_id).copied().unwrap_or_default(),
                generation,
            });
        }
    }
}
