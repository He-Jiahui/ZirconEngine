#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeSnapshot {
    page_table_entry_count: usize,
    resident_page_count: usize,
    pending_request_count: usize,
}

impl VirtualGeometryRuntimeSnapshot {
    pub(in crate::graphics::runtime::virtual_geometry) fn new(
        page_table_entry_count: usize,
        resident_page_count: usize,
        pending_request_count: usize,
    ) -> Self {
        Self {
            page_table_entry_count,
            resident_page_count,
            pending_request_count,
        }
    }

    pub(crate) fn page_table_entry_count(&self) -> usize {
        self.page_table_entry_count
    }

    pub(crate) fn resident_page_count(&self) -> usize {
        self.resident_page_count
    }

    pub(crate) fn pending_request_count(&self) -> usize {
        self.pending_request_count
    }
}
