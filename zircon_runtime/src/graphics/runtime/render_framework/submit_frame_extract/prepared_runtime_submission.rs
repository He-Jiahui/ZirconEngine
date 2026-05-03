#[derive(Default)]
pub(super) struct PreparedRuntimeSubmission {
    hybrid_gi_evictable_probe_ids: Vec<u32>,
    virtual_geometry_evictable_page_ids: Vec<u32>,
}

impl PreparedRuntimeSubmission {
    pub(super) fn new(
        hybrid_gi_evictable_probe_ids: Vec<u32>,
        virtual_geometry_evictable_page_ids: Vec<u32>,
    ) -> Self {
        Self {
            hybrid_gi_evictable_probe_ids,
            virtual_geometry_evictable_page_ids,
        }
    }

    pub(super) fn take_hybrid_gi_evictable_probe_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hybrid_gi_evictable_probe_ids)
    }

    pub(super) fn take_virtual_geometry_evictable_page_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.virtual_geometry_evictable_page_ids)
    }
}
