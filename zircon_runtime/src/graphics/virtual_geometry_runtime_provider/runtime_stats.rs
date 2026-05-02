#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryRuntimeStats {
    page_table_entry_count: usize,
    resident_page_count: usize,
    pending_request_count: usize,
    page_dependency_count: usize,
    completed_page_count: usize,
    replaced_page_count: usize,
}

impl VirtualGeometryRuntimeStats {
    pub fn new(
        page_table_entry_count: usize,
        resident_page_count: usize,
        pending_request_count: usize,
        page_dependency_count: usize,
        completed_page_count: usize,
        replaced_page_count: usize,
    ) -> Self {
        Self {
            page_table_entry_count,
            resident_page_count,
            pending_request_count,
            page_dependency_count,
            completed_page_count,
            replaced_page_count,
        }
    }

    pub fn page_table_entry_count(&self) -> usize {
        self.page_table_entry_count
    }

    pub fn resident_page_count(&self) -> usize {
        self.resident_page_count
    }

    pub fn pending_request_count(&self) -> usize {
        self.pending_request_count
    }

    pub fn page_dependency_count(&self) -> usize {
        self.page_dependency_count
    }

    pub fn completed_page_count(&self) -> usize {
        self.completed_page_count
    }

    pub fn replaced_page_count(&self) -> usize {
        self.replaced_page_count
    }
}
