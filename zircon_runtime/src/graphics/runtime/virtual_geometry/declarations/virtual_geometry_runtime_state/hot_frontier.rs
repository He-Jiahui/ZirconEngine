use std::collections::BTreeSet;

use super::runtime_state::VirtualGeometryRuntimeState;

pub(crate) const HOT_FRONTIER_COOLING_FRAME_COUNT: u8 = 2;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn current_hot_resident_page_ids(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.current_hot_resident_pages.iter().copied()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn recent_hot_resident_page_ids(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.recent_hot_resident_pages.keys().copied()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn insert_current_hot_resident_page(
        &mut self,
        page_id: u32,
    ) -> bool {
        self.current_hot_resident_pages.insert(page_id)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn extend_current_hot_resident_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
    ) {
        self.current_hot_resident_pages.extend(page_ids);
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn extend_recent_hot_resident_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = (u32, u8)>,
    ) {
        self.recent_hot_resident_pages.extend(page_ids);
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn replace_current_hot_resident_pages(
        &mut self,
        page_ids: BTreeSet<u32>,
    ) {
        self.current_hot_resident_pages = page_ids;
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn remove_hot_resident_page(
        &mut self,
        page_id: u32,
    ) {
        self.current_hot_resident_pages.remove(&page_id);
        self.recent_hot_resident_pages.remove(&page_id);
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_current_hot_resident_pages(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.current_hot_resident_pages
            .retain(|page_id| retain(page_id));
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn retain_recent_hot_resident_pages(
        &mut self,
        mut retain: impl FnMut(&u32, &mut u8) -> bool,
    ) {
        self.recent_hot_resident_pages
            .retain(|page_id, frames_remaining| retain(page_id, frames_remaining));
    }
}
