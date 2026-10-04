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
