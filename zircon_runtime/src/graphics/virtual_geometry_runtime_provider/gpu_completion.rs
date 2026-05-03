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
        let completed_page_assignments = outputs
            .completed_page_assignments
            .into_iter()
            .filter_map(|assignment| {
                Some((
                    u32::try_from(assignment.page_id).ok()?,
                    assignment.physical_slot,
                ))
            })
            .collect::<Vec<_>>();
        let completed_page_replacements = outputs
            .page_replacements
            .into_iter()
            .filter_map(|replacement| {
                Some((
                    u32::try_from(replacement.new_page_id).ok()?,
                    u32::try_from(replacement.old_page_id).ok()?,
                ))
            })
            .collect::<Vec<_>>();

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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryPageReplacementRecord,
        RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn gpu_completion_projects_neutral_virtual_geometry_readback_outputs() {
        let completion = VirtualGeometryGpuCompletion::from_readback_outputs(
            RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![20, 2, 30, 3],
                completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                    page_id: 30,
                    physical_slot: 3,
                }],
                page_replacements: vec![RenderVirtualGeometryPageReplacementRecord {
                    old_page_id: 10,
                    new_page_id: 30,
                    physical_slot: 3,
                }],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
        )
        .expect("nonempty readback should create completion");

        assert_eq!(completion.page_table_entries(), &[(20, 2), (30, 3)]);
        assert_eq!(completion.completed_page_assignments(), &[(30, 3)]);
        assert_eq!(completion.completed_page_replacements(), &[(30, 10)]);
    }

    #[test]
    fn gpu_completion_skips_empty_neutral_virtual_geometry_readback_outputs() {
        assert!(VirtualGeometryGpuCompletion::from_readback_outputs(
            RenderVirtualGeometryReadbackOutputs::default()
        )
        .is_none());
    }

    #[test]
    fn gpu_completion_ignores_incomplete_neutral_page_table_pairs() {
        let completion = VirtualGeometryGpuCompletion::from_readback_outputs(
            RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![20, 2, 30],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
        )
        .expect("complete page table pair should create completion");

        assert_eq!(completion.page_table_entries(), &[(20, 2)]);
    }
}
