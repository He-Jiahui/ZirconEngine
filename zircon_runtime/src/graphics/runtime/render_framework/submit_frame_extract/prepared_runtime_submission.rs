#[derive(Default)]
pub(super) struct PreparedRuntimeSubmission {
    virtual_geometry_evictable_page_ids: Vec<u32>,
}

impl PreparedRuntimeSubmission {
    pub(super) fn new(virtual_geometry_evictable_page_ids: Vec<u32>) -> Self {
        Self {
            virtual_geometry_evictable_page_ids,
        }
    }

    pub(super) fn take_virtual_geometry_evictable_page_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.virtual_geometry_evictable_page_ids)
    }
}
