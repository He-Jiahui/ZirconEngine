use std::collections::BTreeMap;

use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry) fn has_page_size(&self, page_id: u32) -> bool {
        self.page_sizes.contains_key(&page_id)
    }

    pub(in crate::virtual_geometry) fn page_size_bytes(&self, page_id: u32) -> u64 {
        self.page_sizes.get(&page_id).copied().unwrap_or_default()
    }

    pub(in crate::virtual_geometry) fn insert_page_size(
        &mut self,
        page_id: u32,
        size_bytes: u64,
    ) -> Option<u64> {
        self.page_sizes.insert(page_id, size_bytes)
    }

    pub(in crate::virtual_geometry) fn retain_page_sizes(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.page_sizes.retain(|page_id, _| retain(page_id));
    }

    pub(in crate::virtual_geometry) fn page_parent_pages(&self) -> &BTreeMap<u32, u32> {
        &self.page_parent_pages
    }

    pub(in crate::virtual_geometry) fn replace_page_parent_pages(
        &mut self,
        page_parent_pages: BTreeMap<u32, u32>,
    ) {
        self.page_parent_pages = page_parent_pages;
    }

    pub(in crate::virtual_geometry) fn retain_page_parent_pages(
        &mut self,
        mut retain: impl FnMut(&u32, &u32) -> bool,
    ) {
        self.page_parent_pages
            .retain(|page_id, parent_page_id| retain(page_id, parent_page_id));
    }
}
