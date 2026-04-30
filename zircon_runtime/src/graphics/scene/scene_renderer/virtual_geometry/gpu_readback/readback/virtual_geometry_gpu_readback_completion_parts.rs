pub(in crate::graphics) struct VirtualGeometryGpuReadbackCompletionParts {
    page_table_entries: Vec<(u32, u32)>,
    completed_page_assignments: Vec<(u32, u32)>,
    completed_page_replacements: Vec<(u32, u32)>,
}

impl VirtualGeometryGpuReadbackCompletionParts {
    pub(super) fn new(
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

    pub(in crate::graphics) fn into_parts(
        self,
    ) -> (Vec<(u32, u32)>, Vec<(u32, u32)>, Vec<(u32, u32)>) {
        (
            self.page_table_entries,
            self.completed_page_assignments,
            self.completed_page_replacements,
        )
    }
}
