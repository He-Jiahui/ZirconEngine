#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryRuntimePrepareOutput {
    evictable_page_ids: Vec<u32>,
}

impl VirtualGeometryRuntimePrepareOutput {
    pub fn new(evictable_page_ids: Vec<u32>) -> Self {
        Self { evictable_page_ids }
    }

    pub fn into_evictable_page_ids(self) -> Vec<u32> {
        self.evictable_page_ids
    }
}
