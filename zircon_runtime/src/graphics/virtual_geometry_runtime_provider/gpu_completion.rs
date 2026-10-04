use crate::core::framework::render::RenderVirtualGeometryReadbackOutputs;

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

    pub(crate) fn from_readback_outputs(
        outputs: RenderVirtualGeometryReadbackOutputs,
    ) -> Option<Self> {
        let page_table_entries =
            page_table_entries_from_neutral_outputs(outputs.page_table_entries);
        let assignment_records = outputs.completed_page_assignments;
        let mut completed_page_assignments = Vec::with_capacity(assignment_records.len());
        for assignment in assignment_records {
            let Ok(page_id) = u32::try_from(assignment.page_id) else {
                continue;
            };
            completed_page_assignments.push((page_id, assignment.physical_slot));
        }
        let replacement_records = outputs.page_replacements;
        let mut completed_page_replacements = Vec::with_capacity(replacement_records.len());
        for replacement in replacement_records {
            let (Ok(new_page_id), Ok(old_page_id)) = (
                u32::try_from(replacement.new_page_id),
                u32::try_from(replacement.old_page_id),
            ) else {
                continue;
            };
            completed_page_replacements.push((new_page_id, old_page_id));
        }

        if page_table_entries.is_empty()
            && completed_page_assignments.is_empty()
            && completed_page_replacements.is_empty()
        {
            return None;
        }

        Some(Self::new(
            page_table_entries,
            completed_page_assignments,
            completed_page_replacements,
        ))
    }
}

fn page_table_entries_from_neutral_outputs(entries: Vec<u32>) -> Vec<(u32, u32)> {
    entries
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect()
}

#[cfg(test)]
#[path = "tests/gpu_completion.rs"]
mod tests;
