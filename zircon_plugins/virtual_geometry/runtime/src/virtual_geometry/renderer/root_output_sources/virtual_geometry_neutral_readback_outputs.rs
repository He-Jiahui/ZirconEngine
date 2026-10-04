use std::collections::HashMap;

use crate::virtual_geometry::renderer::VirtualGeometryGpuReadback;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryPageReplacementRecord,
    RenderVirtualGeometryReadbackOutputs,
};

impl From<VirtualGeometryGpuReadback> for RenderVirtualGeometryReadbackOutputs {
    fn from(readback: VirtualGeometryGpuReadback) -> Self {
        Self {
            page_table_entries: flat_page_table_entries(readback.page_table_entries()),
            completed_page_assignments: completed_page_assignments(
                readback.completed_page_assignments(),
            ),
            page_replacements: page_replacements(
                readback.completed_page_replacements(),
                readback.page_table_entries(),
                readback.completed_page_assignments(),
            ),
            selected_clusters: readback.selected_clusters().to_vec(),
            visbuffer64_entries: readback.visbuffer64_entries().to_vec(),
            hardware_rasterization_records: readback.hardware_rasterization_records().to_vec(),
            node_cluster_cull: readback.node_cluster_cull().clone(),
            ..RenderVirtualGeometryReadbackOutputs::default()
        }
    }
}

fn flat_page_table_entries(entries: &[(u32, u32)]) -> Vec<u32> {
    let mut flat_entries = Vec::with_capacity(entries.len().saturating_mul(2));
    for &(page_id, slot) in entries {
        flat_entries.extend_from_slice(&[page_id, slot]);
    }
    flat_entries
}

fn completed_page_assignments(
    assignments: &[(u32, u32)],
) -> Vec<RenderVirtualGeometryPageAssignmentRecord> {
    assignments
        .iter()
        .map(
            |&(page_id, physical_slot)| RenderVirtualGeometryPageAssignmentRecord {
                page_id: u64::from(page_id),
                physical_slot,
            },
        )
        .collect()
}

// 替换事件仅携带新旧页 ID；优先用完成分配、再用页表为新页补出物理槽，缺失时保留默认槽值。
fn page_replacements(
    replacements: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
    completed_page_assignments: &[(u32, u32)],
) -> Vec<RenderVirtualGeometryPageReplacementRecord> {
    let mut slots_by_page_id = HashMap::with_capacity(
        completed_page_assignments
            .len()
            .saturating_add(page_table_entries.len()),
    );
    for &(page_id, slot) in completed_page_assignments
        .iter()
        .chain(page_table_entries.iter())
    {
        slots_by_page_id.entry(page_id).or_insert(slot);
    }

    replacements
        .iter()
        .map(
            |&(new_page_id, old_page_id)| RenderVirtualGeometryPageReplacementRecord {
                old_page_id: u64::from(old_page_id),
                new_page_id: u64::from(new_page_id),
                physical_slot: slots_by_page_id
                    .get(&new_page_id)
                    .copied()
                    .unwrap_or_default(),
            },
        )
        .collect()
}

#[cfg(test)]
#[path = "virtual_geometry_neutral_readback_outputs/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/virtual_geometry_neutral_readback_outputs.rs"]
mod tests;
