use std::collections::{BTreeMap, BTreeSet};

use super::super::virtual_geometry_page_request::VirtualGeometryPageRequest;
use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn clear_current_requested_page_order(
        &mut self,
    ) {
        self.current_requested_page_order.clear();
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn current_requested_page_order_len(
        &self,
    ) -> usize {
        self.current_requested_page_order.len()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn current_requested_page_order(
        &self,
    ) -> &BTreeMap<u32, usize> {
        &self.current_requested_page_order
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn current_request_rank(
        &self,
        page_id: u32,
    ) -> usize {
        self.current_requested_page_order
            .get(&page_id)
            .copied()
            .unwrap_or(usize::MAX)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn ensure_current_requested_page_order(
        &mut self,
        page_id: u32,
        order: usize,
    ) -> bool {
        if self.current_requested_page_order.contains_key(&page_id) {
            return false;
        }
        self.current_requested_page_order.insert(page_id, order);
        true
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_current_requested_page_order(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.current_requested_page_order
            .retain(|page_id, _| retain(page_id));
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn pending_page_requests(
        &self,
    ) -> &[VirtualGeometryPageRequest] {
        &self.pending_requests
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn push_pending_page_request(
        &mut self,
        request: VirtualGeometryPageRequest,
    ) {
        self.pending_requests.push(request);
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_pending_page_requests(
        &mut self,
        mut retain: impl FnMut(&VirtualGeometryPageRequest) -> bool,
    ) {
        self.pending_requests.retain(|request| retain(request));
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn has_pending_page(
        &self,
        page_id: u32,
    ) -> bool {
        self.pending_pages.contains(&page_id)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn pending_page_id_iter(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.pending_pages.iter().copied()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn insert_pending_page(
        &mut self,
        page_id: u32,
    ) -> bool {
        self.pending_pages.insert(page_id)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn remove_pending_page(
        &mut self,
        page_id: u32,
    ) -> bool {
        self.pending_pages.remove(&page_id)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_pending_pages(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.pending_pages.retain(|page_id| retain(page_id));
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn evictable_page_ids(&self) -> &[u32] {
        &self.evictable_pages
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn clear_evictable_pages(&mut self) {
        self.evictable_pages.clear();
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn replace_evictable_pages(
        &mut self,
        evictable_pages: Vec<u32>,
    ) {
        self.evictable_pages = evictable_pages;
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_evictable_pages(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.evictable_pages.retain(|page_id| retain(page_id));
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn remove_evictable_page(
        &mut self,
        page_id: u32,
    ) {
        self.retain_evictable_pages(|candidate| *candidate != page_id);
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_resident_evictable_pages(
        &mut self,
    ) {
        let resident_page_ids = self.resident_page_ids().collect::<BTreeSet<_>>();
        self.retain_evictable_pages(|page_id| resident_page_ids.contains(page_id));
    }
}
