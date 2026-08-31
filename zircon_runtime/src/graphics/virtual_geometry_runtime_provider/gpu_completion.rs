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
        assert!(
            VirtualGeometryGpuCompletion::from_readback_outputs(
                RenderVirtualGeometryReadbackOutputs::default()
            )
            .is_none()
        );
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

    #[test]
    fn gpu_completion_preallocates_filtered_record_projections() {
        let source = include_str!("gpu_completion.rs");
        let assignments = concat!("Vec::with_capacity(", "assignment_records.len())");
        let replacements = concat!("Vec::with_capacity(", "replacement_records.len())");

        assert!(source.contains(assignments));
        assert!(source.contains(replacements));
    }

    #[test]
    fn gpu_completion_skips_records_outside_runtime_page_id_range() {
        let overflow = u64::from(u32::MAX) + 1;
        let completion = VirtualGeometryGpuCompletion::from_readback_outputs(
            RenderVirtualGeometryReadbackOutputs {
                completed_page_assignments: vec![
                    RenderVirtualGeometryPageAssignmentRecord {
                        page_id: 30,
                        physical_slot: 3,
                    },
                    RenderVirtualGeometryPageAssignmentRecord {
                        page_id: overflow,
                        physical_slot: 4,
                    },
                ],
                page_replacements: vec![
                    RenderVirtualGeometryPageReplacementRecord {
                        old_page_id: 10,
                        new_page_id: 30,
                        physical_slot: 3,
                    },
                    RenderVirtualGeometryPageReplacementRecord {
                        old_page_id: overflow,
                        new_page_id: 40,
                        physical_slot: 4,
                    },
                ],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
        )
        .expect("valid page records should keep completion");

        assert_eq!(completion.completed_page_assignments(), &[(30, 3)]);
        assert_eq!(completion.completed_page_replacements(), &[(30, 10)]);
    }
}
