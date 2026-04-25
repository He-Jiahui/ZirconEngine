use super::*;
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};

pub(in super::super) fn selection(
    instance_index: Option<u32>,
    entity: u64,
    submission_index: u32,
    cluster_id: u32,
    cluster_ordinal: u32,
    page_id: u32,
    lod_level: u8,
    state: VirtualGeometryPrepareClusterState,
) -> VirtualGeometryClusterSelection {
    VirtualGeometryClusterSelection {
        submission_index,
        instance_index,
        entity,
        cluster_id,
        cluster_ordinal,
        page_id,
        lod_level,
        submission_page_id: page_id,
        submission_lod_level: lod_level,
        entity_cluster_start_ordinal: cluster_ordinal as usize,
        entity_cluster_span_count: 1,
        entity_cluster_total_count: 3,
        lineage_depth: 0,
        frontier_rank: 0,
        resident_slot: Some(0),
        submission_slot: Some(0),
        state,
    }
}

pub(in super::super) fn clusters_by_id(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<u32, RenderVirtualGeometryCluster> {
    extract
        .clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect()
}

pub(in super::super) fn cluster_ordering(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    seed_backed_cluster_ordering(extract)
}

pub(in super::super) fn node_and_cluster_cull_pass_output_from_launch_worklist(
    cluster_budget: u32,
    page_budget: u32,
    instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    debug: RenderVirtualGeometryDebugState,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    let instance_seed_count = u32::try_from(instance_seeds.len()).unwrap_or(u32::MAX);
    let global_state = RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
        cull_input: RenderVirtualGeometryCullInputSnapshot {
            cluster_budget,
            page_budget,
            instance_count: instance_seed_count,
            cluster_count: cluster_budget,
            page_count: page_budget,
            visible_entity_count: instance_seed_count,
            visible_cluster_count: cluster_budget,
            resident_page_count: 0,
            pending_page_request_count: 0,
            available_page_slot_count: 0,
            evictable_page_count: 0,
            debug,
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        },
        viewport_size: [96, 64],
        camera_translation: [0.0, 0.0, 0.0],
        child_split_screen_space_error_threshold: 1.0,
        child_frustum_culling_enabled: true,
        view_proj: [[0.0; 4]; 4],
        previous_camera_translation: [0.0, 0.0, 0.0],
        previous_view_proj: [[0.0; 4]; 4],
    };
    let dispatch_setup = RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
        instance_seed_count,
        cluster_budget,
        page_budget,
        workgroup_size: 64,
        dispatch_group_count: [instance_seed_count.div_ceil(64), 1, 1],
    };
    let launch_worklist = RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
        global_state: global_state.clone(),
        dispatch_setup,
        instance_seeds: instance_seeds.clone(),
    };
    let instance_work_items = instance_seeds
        .iter()
        .map(
            |seed| RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
                instance_index: seed.instance_index,
                entity: seed.entity,
                cluster_offset: seed.cluster_offset,
                cluster_count: seed.cluster_count,
                page_offset: seed.page_offset,
                page_count: seed.page_count,
                cluster_budget,
                page_budget,
                forced_mip: debug.forced_mip,
            },
        )
        .collect::<Vec<_>>();
    let instance_work_item_count = u32::try_from(instance_work_items.len()).unwrap_or(u32::MAX);
    let cluster_work_items = instance_work_items
        .iter()
        .flat_map(|work_item| {
            (0..work_item.cluster_count).map(move |cluster_local_index| {
                let cluster_array_index =
                    work_item.cluster_offset.saturating_add(cluster_local_index);
                VirtualGeometryNodeAndClusterCullClusterWorkItem {
                    instance_index: work_item.instance_index,
                    entity: work_item.entity,
                    cluster_array_index,
                    hierarchy_node_id: None,
                    cluster_budget: work_item.cluster_budget,
                    page_budget: work_item.page_budget,
                    forced_mip: work_item.forced_mip,
                }
            })
        })
        .collect::<Vec<_>>();
    let mut traversal_records = Vec::new();
    let mut traversal_index = 0u32;
    let mut stored_cluster_count = 0u32;
    for work_item in &cluster_work_items {
        traversal_records.push(VirtualGeometryNodeAndClusterCullTraversalRecord {
            op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
            child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
            instance_index: work_item.instance_index,
            entity: work_item.entity,
            cluster_array_index: work_item.cluster_array_index,
            hierarchy_node_id: None,
            node_cluster_start: 0,
            node_cluster_count: 0,
            child_base: 0,
            child_count: 0,
            traversal_index,
            cluster_budget: work_item.cluster_budget,
            page_budget: work_item.page_budget,
            forced_mip: work_item.forced_mip,
        });
        traversal_index = traversal_index.saturating_add(1);
        if stored_cluster_count < work_item.cluster_budget {
            traversal_records.push(VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: work_item.instance_index,
                entity: work_item.entity,
                cluster_array_index: work_item.cluster_array_index,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index,
                cluster_budget: work_item.cluster_budget,
                page_budget: work_item.page_budget,
                forced_mip: work_item.forced_mip,
            });
            traversal_index = traversal_index.saturating_add(1);
            stored_cluster_count = stored_cluster_count.saturating_add(1);
        } else {
            traversal_records.push(VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                child_source:
                    VirtualGeometryNodeAndClusterCullTraversalChildSource::CompatFixedFanout,
                instance_index: work_item.instance_index,
                entity: work_item.entity,
                cluster_array_index: work_item.cluster_array_index,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: work_item.cluster_array_index.saturating_mul(4),
                child_count: 4,
                traversal_index,
                cluster_budget: work_item.cluster_budget,
                page_budget: work_item.page_budget,
                forced_mip: work_item.forced_mip,
            });
            traversal_index = traversal_index.saturating_add(1);
        }
    }

    VirtualGeometryNodeAndClusterCullPassOutput {
        source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        record_count: 1,
        global_state: Some(global_state),
        buffer: None,
        dispatch_setup: Some(dispatch_setup),
        launch_worklist: Some(launch_worklist),
        dispatch_setup_buffer: None,
        launch_worklist_buffer: None,
        instance_seed_count,
        instance_seeds,
        instance_seed_buffer: None,
        instance_work_item_count,
        instance_work_items,
        instance_work_item_buffer: None,
        cluster_work_item_count: u32::try_from(cluster_work_items.len()).unwrap_or(u32::MAX),
        cluster_work_items,
        cluster_work_item_buffer: None,
        hierarchy_child_ids: Vec::new(),
        hierarchy_child_id_buffer: None,
        child_work_item_count: 0,
        child_work_items: Vec::new(),
        child_work_item_buffer: None,
        traversal_record_count: u32::try_from(traversal_records.len()).unwrap_or(u32::MAX),
        traversal_records,
        traversal_record_buffer: None,
        page_request_count: 0,
        page_request_ids: Vec::new(),
        page_request_buffer: None,
    }
}
