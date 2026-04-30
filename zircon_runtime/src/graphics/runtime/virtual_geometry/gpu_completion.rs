pub(in crate::graphics::runtime) struct VirtualGeometryGpuCompletion {
    page_table_entries: Vec<(u32, u32)>,
    completed_page_assignments: Vec<(u32, u32)>,
    completed_page_replacements: Vec<(u32, u32)>,
}

impl VirtualGeometryGpuCompletion {
    pub(in crate::graphics::runtime) fn new(
        page_table_entries: Vec<(u32, u32)>,
        completed_page_assignments: Vec<(u32, u32)>,
        completed_page_replacements: Vec<(u32, u32)>,
    ) -> Self {
        Self {
            page_table_entries,
            completed_page_assignments,
            completed_page_replacements,
        }
    }

    pub(in crate::graphics::runtime) fn page_table_entries(&self) -> &[(u32, u32)] {
        &self.page_table_entries
    }

    pub(in crate::graphics::runtime) fn completed_page_assignments(&self) -> &[(u32, u32)] {
        &self.completed_page_assignments
    }

    pub(in crate::graphics::runtime) fn completed_page_replacements(&self) -> &[(u32, u32)] {
        &self.completed_page_replacements
    }
}
