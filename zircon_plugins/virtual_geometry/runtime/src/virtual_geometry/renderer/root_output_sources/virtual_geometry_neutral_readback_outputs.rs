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
    entries
        .iter()
        .flat_map(|&(page_id, slot)| [page_id, slot])
        .collect()
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

fn page_replacements(
    replacements: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
    completed_page_assignments: &[(u32, u32)],
) -> Vec<RenderVirtualGeometryPageReplacementRecord> {
    replacements
        .iter()
        .map(
            |&(new_page_id, old_page_id)| RenderVirtualGeometryPageReplacementRecord {
                old_page_id: u64::from(old_page_id),
                new_page_id: u64::from(new_page_id),
                physical_slot: slot_for_new_page(
                    new_page_id,
                    completed_page_assignments,
                    page_table_entries,
                ),
            },
        )
        .collect()
}

fn slot_for_new_page(
    page_id: u32,
    completed_page_assignments: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
) -> u32 {
    completed_page_assignments
        .iter()
        .chain(page_table_entries.iter())
        .find_map(|&(candidate_page_id, slot)| (candidate_page_id == page_id).then_some(slot))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
        RenderVirtualGeometryHardwareRasterizationSource,
        RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
        RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
        RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
        RenderVirtualGeometryNodeAndClusterCullTraversalOp,
        RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
    };

    #[test]
    fn neutral_outputs_project_virtual_geometry_gpu_readback() {
        let mut readback = VirtualGeometryGpuReadback::new(
            vec![(20, 2), (30, 3)],
            vec![20, 30],
            vec![(30, 3)],
            vec![(30, 10)],
        );
        let hardware_records = vec![hardware_record()];
        let node_cluster_cull = node_cluster_cull_outputs();
        readback.replace_render_path_readback(
            1,
            RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
            hardware_records.clone(),
            0,
            RenderVirtualGeometrySelectedClusterSource::Unavailable,
            Vec::new(),
            0,
            RenderVirtualGeometryVisBuffer64Source::Unavailable,
            0,
            Vec::new(),
        );
        readback.replace_node_cluster_cull_readback(node_cluster_cull.clone());

        let outputs = RenderVirtualGeometryReadbackOutputs::from(readback);

        assert_eq!(outputs.page_table_entries, vec![20, 2, 30, 3]);
        assert_eq!(
            outputs.completed_page_assignments,
            vec![RenderVirtualGeometryPageAssignmentRecord {
                page_id: 30,
                physical_slot: 3,
            }]
        );
        assert_eq!(
            outputs.page_replacements,
            vec![RenderVirtualGeometryPageReplacementRecord {
                old_page_id: 10,
                new_page_id: 30,
                physical_slot: 3,
            }]
        );
        assert_eq!(outputs.hardware_rasterization_records, hardware_records);
        assert_eq!(outputs.node_cluster_cull, node_cluster_cull);
        assert_eq!(outputs.node_cluster_cull.page_request_ids, vec![300, 301]);
    }

    #[test]
    fn neutral_outputs_stay_empty_without_virtual_geometry_gpu_readback_payload() {
        let outputs = RenderVirtualGeometryReadbackOutputs::from(VirtualGeometryGpuReadback::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));

        assert_eq!(outputs, RenderVirtualGeometryReadbackOutputs::default());
    }

    fn hardware_record() -> RenderVirtualGeometryHardwareRasterizationRecord {
        RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: 77,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            submission_index: 2,
            submission_page_id: 300,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: 1,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 4,
            resident_slot: Some(3),
            submission_slot: Some(5),
            state: RenderVirtualGeometryExecutionState::Resident,
        }
    }

    fn node_cluster_cull_outputs() -> RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        RenderVirtualGeometryNodeClusterCullReadbackOutputs {
            traversal_records: vec![RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
                op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 77,
                cluster_array_index: 1,
                hierarchy_node_id: Some(4),
                node_cluster_start: 1,
                node_cluster_count: 1,
                child_base: 0,
                child_count: 0,
                traversal_index: 2,
                cluster_budget: 8,
                page_budget: 4,
                forced_mip: None,
            }],
            child_work_items: vec![RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
                instance_index: 0,
                entity: 77,
                parent_cluster_array_index: 1,
                parent_hierarchy_node_id: Some(4),
                child_node_id: 9,
                child_table_index: 0,
                traversal_index: 2,
                cluster_budget: 8,
                page_budget: 4,
                forced_mip: Some(1),
            }],
            cluster_work_items: vec![RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index: 0,
                entity: 77,
                cluster_array_index: 1,
                hierarchy_node_id: Some(4),
                cluster_budget: 8,
                page_budget: 4,
                forced_mip: None,
            }],
            launch_worklist_snapshots: Vec::new(),
            page_request_ids: vec![300, 301],
        }
    }
}
