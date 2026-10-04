use super::*;
use crate::core::framework::render::{
    RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryPageReplacementRecord,
    RenderVirtualGeometryReadbackOutputs,
};

#[test]
fn gpu_completion_projects_neutral_virtual_geometry_readback_outputs() {
    let completion =
        VirtualGeometryGpuCompletion::from_readback_outputs(RenderVirtualGeometryReadbackOutputs {
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
        })
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
    let completion =
        VirtualGeometryGpuCompletion::from_readback_outputs(RenderVirtualGeometryReadbackOutputs {
            page_table_entries: vec![20, 2, 30],
            ..RenderVirtualGeometryReadbackOutputs::default()
        })
        .expect("complete page table pair should create completion");

    assert_eq!(completion.page_table_entries(), &[(20, 2)]);
}

#[test]
fn gpu_completion_preallocates_filtered_record_projections() {
    let source = include_str!("../gpu_completion.rs");
    let assignments = concat!("Vec::with_capacity(", "assignment_records.len())");
    let replacements = concat!("Vec::with_capacity(", "replacement_records.len())");

    assert!(source.contains(assignments));
    assert!(source.contains(replacements));
}

#[test]
fn gpu_completion_skips_records_outside_runtime_page_id_range() {
    let overflow = u64::from(u32::MAX) + 1;
    let completion =
        VirtualGeometryGpuCompletion::from_readback_outputs(RenderVirtualGeometryReadbackOutputs {
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
        })
        .expect("valid page records should keep completion");

    assert_eq!(completion.completed_page_assignments(), &[(30, 3)]);
    assert_eq!(completion.completed_page_replacements(), &[(30, 10)]);
}
