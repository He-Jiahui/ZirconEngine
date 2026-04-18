use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry) fn evict_one(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
    ) -> bool {
        for page_id in page_ids {
            if self.evict_page(page_id).is_some() {
                return true;
            }
        }
        false
    }
}
