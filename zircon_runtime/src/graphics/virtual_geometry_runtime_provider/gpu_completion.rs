#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryGpuCompletion {
    page_table_entries: Vec<(u32, u32)>,
    completed_page_assignments: Vec<(u32, u32)>,
    completed_page_replacements: Vec<(u32, u32)>,
}

impl VirtualGeometryGpuCompletion {
    pub fn new(
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

    pub fn page_table_entries(&self) -> &[(u32, u32)] {
        &self.page_table_entries
    }

    pub fn completed_page_assignments(&self) -> &[(u32, u32)] {
        &self.completed_page_assignments
    }

    pub fn completed_page_replacements(&self) -> &[(u32, u32)] {
        &self.completed_page_replacements
    }
}
