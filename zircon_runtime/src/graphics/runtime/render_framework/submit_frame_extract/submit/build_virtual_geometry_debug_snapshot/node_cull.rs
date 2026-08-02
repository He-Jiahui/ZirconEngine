use std::collections::BTreeSet;

use super::super::super::frame_submission_context::FrameSubmissionContext;
use super::support::saturated_u32_len;
use crate::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};
use crate::core::math::{view_matrix, Mat4};
use crate::graphics::VisibilityViewKey;

pub(super) struct NodeAndClusterCullSnapshot {
    pub(super) source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub(super) record_count: u32,
    pub(super) global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(super) dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(super) launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub(super) instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(super) instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub(super) cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub(super) child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub(super) traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(super) page_request_ids: Vec<u32>,
}

pub(super) fn build_node_and_cluster_cull_snapshot(
    frame_extract: &RenderFrameExtract,
    context: &FrameSubmissionContext,
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> NodeAndClusterCullSnapshot {
    let Some(extract) = context.virtual_geometry_extract() else {
        return empty_node_and_cluster_cull_snapshot();
    };
    if extract.instances.is_empty() {
        return empty_node_and_cluster_cull_snapshot();
    }

    let global_state = build_node_and_cluster_cull_global_state(frame_extract, context, cull_input);
    let instance_seeds = build_node_and_cluster_cull_instance_seeds(extract);
    let dispatch_setup = RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
        instance_seed_count: saturated_u32_len(instance_seeds.len()),
        cluster_budget: cull_input.cluster_budget,
        page_budget: cull_input.page_budget,
        workgroup_size: 64,
        dispatch_group_count: [
            saturated_u32_len(instance_seeds.len()).max(1).div_ceil(64),
            1,
            1,
        ],
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
                cluster_budget: cull_input.cluster_budget,
                page_budget: cull_input.page_budget,
                forced_mip: cull_input.debug.forced_mip,
            },
        )
        .collect::<Vec<_>>();
    let cluster_work_items = build_node_and_cluster_cull_cluster_work_items(extract, cull_input);
    let (child_work_items, traversal_records) =
        build_node_and_cluster_cull_traversal_records(extract, &cluster_work_items, cull_input);
    let page_request_ids =
        build_node_and_cluster_cull_page_request_ids(extract, &traversal_records, cull_input);

    NodeAndClusterCullSnapshot {
        source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        record_count: 1,
        global_state: Some(global_state),
        dispatch_setup: Some(dispatch_setup),
        launch_worklist: Some(launch_worklist),
        instance_seeds,
        instance_work_items,
        cluster_work_items,
        child_work_items,
        traversal_records,
        page_request_ids,
    }
}

fn empty_node_and_cluster_cull_snapshot() -> NodeAndClusterCullSnapshot {
    NodeAndClusterCullSnapshot {
        source: RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
        record_count: 0,
        global_state: None,
        dispatch_setup: None,
        launch_worklist: None,
        instance_seeds: Vec::new(),
        instance_work_items: Vec::new(),
        cluster_work_items: Vec::new(),
        child_work_items: Vec::new(),
        traversal_records: Vec::new(),
        page_request_ids: Vec::new(),
    }
}

fn build_node_and_cluster_cull_global_state(
    frame_extract: &RenderFrameExtract,
    context: &FrameSubmissionContext,
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
    let camera = context
        .view_visibility(&VisibilityViewKey::MainCamera)
        .map(|view| &view.camera)
        .unwrap_or(&frame_extract.view.camera);
    let size = context.size();
    let aspect = size.x.max(1) as f32 / size.y.max(1) as f32;
    let view_proj = Mat4::perspective_rh(camera.fov_y_radians, aspect, camera.z_near, camera.z_far)
        .mul_mat4(&view_matrix(camera.transform))
        .to_cols_array_2d();

    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
        cull_input,
        viewport_size: [size.x, size.y],
        camera_translation: camera.transform.translation.to_array(),
        child_split_screen_space_error_threshold: 64.0 / size.y.max(1) as f32,
        child_frustum_culling_enabled: true,
        view_proj,
        previous_camera_translation: camera.transform.translation.to_array(),
        previous_view_proj: view_proj,
    }
}

fn build_node_and_cluster_cull_instance_seeds(
    extract: &RenderVirtualGeometryExtract,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed> {
    extract
        .instances
        .iter()
        .enumerate()
        .map(
            |(instance_index, instance)| RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: u32::try_from(instance_index).unwrap_or(u32::MAX),
                entity: instance.entity,
                cluster_offset: instance.cluster_offset,
                cluster_count: instance.cluster_count,
                page_offset: instance.page_offset,
                page_count: instance.page_count,
            },
        )
        .collect()
}

fn build_node_and_cluster_cull_cluster_work_items(
    extract: &RenderVirtualGeometryExtract,
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem> {
    let mut work_items = Vec::new();
    for (instance_index, instance) in extract.instances.iter().enumerate() {
        let instance_index = u32::try_from(instance_index).unwrap_or(u32::MAX);
        for cluster_array_index in instance.cluster_offset
            ..instance
                .cluster_offset
                .saturating_add(instance.cluster_count)
        {
            let hierarchy_node_id = extract
                .clusters
                .get(cluster_array_index as usize)
                .and_then(|cluster| cluster.hierarchy_node_id);
            work_items.push(RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
                instance_index,
                entity: instance.entity,
                cluster_array_index,
                hierarchy_node_id,
                cluster_budget: cull_input.cluster_budget,
                page_budget: cull_input.page_budget,
                forced_mip: cull_input.debug.forced_mip,
            });
        }
    }
    work_items
}

fn build_node_and_cluster_cull_traversal_records(
    extract: &RenderVirtualGeometryExtract,
    cluster_work_items: &[RenderVirtualGeometryNodeAndClusterCullClusterWorkItem],
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> (
    Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
) {
    let mut child_work_items = Vec::new();
    let mut traversal_records = Vec::new();
    let mut queue = cluster_work_items
        .iter()
        .map(|work_item| TraversalQueueItem {
            instance_index: work_item.instance_index,
            entity: work_item.entity,
            cluster_array_index: work_item.cluster_array_index,
            hierarchy_node_id: work_item.hierarchy_node_id,
        })
        .collect::<Vec<_>>();
    let mut cursor = 0;

    while cursor < queue.len() {
        let item = queue[cursor];
        cursor += 1;
        let node = item
            .hierarchy_node_id
            .and_then(|node_id| hierarchy_node(extract, node_id));
        push_traversal_record(
            &mut traversal_records,
            RenderVirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
            RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None,
            item,
            node,
            cull_input,
        );

        if let Some(node) = node.filter(|node| node.child_count > 0) {
            for child_table_index in node.child_base..node.child_base + node.child_count {
                let child_node_id = extract
                    .hierarchy_child_ids
                    .get(child_table_index as usize)
                    .copied()
                    .unwrap_or(u32::MAX);
                push_traversal_record(
                    &mut traversal_records,
                    RenderVirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                    item,
                    Some(node),
                    cull_input,
                );
                let traversal_index =
                    u32::try_from(traversal_records.len().saturating_sub(1)).unwrap_or(u32::MAX);
                child_work_items.push(RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
                    instance_index: item.instance_index,
                    entity: item.entity,
                    parent_cluster_array_index: item.cluster_array_index,
                    parent_hierarchy_node_id: item.hierarchy_node_id,
                    child_node_id,
                    child_table_index,
                    traversal_index,
                    cluster_budget: cull_input.cluster_budget,
                    page_budget: cull_input.page_budget,
                    forced_mip: cull_input.debug.forced_mip,
                });
                queue.push(TraversalQueueItem {
                    instance_index: item.instance_index,
                    entity: item.entity,
                    cluster_array_index: item.cluster_array_index,
                    hierarchy_node_id: Some(child_node_id),
                });
            }
            continue;
        }

        let store_item = node
            .filter(|node| node.cluster_count > 0)
            .map(|node| TraversalQueueItem {
                cluster_array_index: node.cluster_start,
                hierarchy_node_id: Some(node.node_id),
                ..item
            })
            .unwrap_or(item);
        push_traversal_record(
            &mut traversal_records,
            RenderVirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
            RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None,
            store_item,
            node,
            cull_input,
        );
    }

    (child_work_items, traversal_records)
}

fn build_node_and_cluster_cull_page_request_ids(
    extract: &RenderVirtualGeometryExtract,
    traversal_records: &[RenderVirtualGeometryNodeAndClusterCullTraversalRecord],
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) -> Vec<u32> {
    let resident_page_ids = extract
        .pages
        .iter()
        .filter(|page| page.resident)
        .map(|page| page.page_id)
        .collect::<BTreeSet<_>>();
    let mut page_request_ids = BTreeSet::new();

    for record in traversal_records.iter().filter(|record| {
        record.op == RenderVirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
    }) {
        let cluster_start = if record.node_cluster_count > 0 {
            record.node_cluster_start
        } else {
            record.cluster_array_index
        };
        let cluster_count = record.node_cluster_count.max(1);

        for cluster_array_index in cluster_start..cluster_start.saturating_add(cluster_count) {
            let Some(cluster) = extract.clusters.get(cluster_array_index as usize) else {
                continue;
            };
            if !resident_page_ids.contains(&cluster.page_id) {
                page_request_ids.insert(cluster.page_id);
            }
        }
    }

    page_request_ids
        .into_iter()
        .take(cull_input.page_budget as usize)
        .collect()
}

#[derive(Clone, Copy)]
struct TraversalQueueItem {
    instance_index: u32,
    entity: u64,
    cluster_array_index: u32,
    hierarchy_node_id: Option<u32>,
}

fn hierarchy_node(
    extract: &RenderVirtualGeometryExtract,
    node_id: u32,
) -> Option<&RenderVirtualGeometryHierarchyNode> {
    extract
        .hierarchy_nodes
        .iter()
        .find(|node| node.node_id == node_id)
}

fn push_traversal_record(
    records: &mut Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    op: RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    child_source: RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    item: TraversalQueueItem,
    node: Option<&RenderVirtualGeometryHierarchyNode>,
    cull_input: RenderVirtualGeometryCullInputSnapshot,
) {
    records.push(RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
        op,
        child_source,
        instance_index: item.instance_index,
        entity: item.entity,
        cluster_array_index: item.cluster_array_index,
        hierarchy_node_id: item.hierarchy_node_id,
        node_cluster_start: node.map(|node| node.cluster_start).unwrap_or(0),
        node_cluster_count: node.map(|node| node.cluster_count).unwrap_or(0),
        child_base: node.map(|node| node.child_base).unwrap_or(0),
        child_count: node.map(|node| node.child_count).unwrap_or(0),
        traversal_index: saturated_u32_len(records.len()),
        cluster_budget: cull_input.cluster_budget,
        page_budget: cull_input.page_budget,
        forced_mip: cull_input.debug.forced_mip,
    });
}
