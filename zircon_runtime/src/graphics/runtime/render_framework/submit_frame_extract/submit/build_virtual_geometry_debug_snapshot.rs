use std::collections::BTreeSet;

use super::super::frame_submission_context::FrameSubmissionContext;
use crate::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
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
    RenderVirtualGeometryPageRequestInspection, RenderVirtualGeometryResidentPageInspection,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderVirtualGeometryVisBufferMark,
};
use crate::core::math::{view_matrix, Mat4};
use crate::graphics::{
    VisibilityVirtualGeometryDrawSegment, VisibilityVirtualGeometryPageUploadPlan,
};

pub(super) fn build_virtual_geometry_debug_snapshot(
    frame_extract: &RenderFrameExtract,
    context: &FrameSubmissionContext,
) -> Option<RenderVirtualGeometryDebugSnapshot> {
    let extract = context.virtual_geometry_extract()?;
    let page_upload_plan = context
        .virtual_geometry_page_upload_plan()
        .cloned()
        .unwrap_or_default();
    let feedback = context
        .virtual_geometry_feedback()
        .cloned()
        .unwrap_or_default();
    let visible_cluster_ids = feedback.visible_cluster_ids.clone();
    let visible_cluster_id_set = visible_cluster_ids.iter().copied().collect::<BTreeSet<_>>();
    let resident_page_set = page_upload_plan
        .resident_pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let requested_page_set = page_upload_plan
        .requested_pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let resident_page_inspections = build_resident_page_inspections(extract, &page_upload_plan);
    let available_page_slots = build_available_page_slots(extract, &page_upload_plan);
    let pending_page_request_inspections = build_pending_page_request_inspections(
        extract,
        context,
        &page_upload_plan,
        &available_page_slots,
    );
    let evictable_page_inspections =
        build_evictable_page_inspections(extract, &page_upload_plan, &resident_page_inspections);
    let leaf_clusters = extract
        .debug
        .print_leaf_clusters
        .then(|| {
            extract
                .clusters
                .iter()
                .copied()
                .filter(|cluster| visible_cluster_id_set.contains(&cluster.cluster_id))
                .collect()
        })
        .unwrap_or_default();
    let bvh_visualization_instances = extract
        .debug
        .visualize_bvh
        .then(|| {
            context
                .virtual_geometry_bvh_visualization_instances()
                .to_vec()
        })
        .unwrap_or_default();
    let selected_clusters = build_selected_clusters_from_visibility_feedback(
        extract,
        &visible_cluster_id_set,
        &resident_page_set,
        &requested_page_set,
    );
    let visbuffer_debug_marks = extract
        .debug
        .visualize_visbuffer
        .then(|| build_visbuffer_debug_marks_from_selected_clusters(&selected_clusters))
        .unwrap_or_default();
    let visbuffer64_entries = build_visbuffer64_entries_from_selected_clusters(&selected_clusters);
    let cull_input = build_cull_input_snapshot(
        extract,
        &page_upload_plan,
        available_page_slots.len(),
        evictable_page_inspections.len(),
    );
    let node_and_cluster_cull =
        build_node_and_cluster_cull_snapshot(frame_extract, context, cull_input);
    let execution = build_execution_snapshot(
        context
            .visibility_context()
            .virtual_geometry_draw_segments
            .as_slice(),
        &resident_page_set,
        &requested_page_set,
    );

    Some(RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        page_dependencies: extract.page_dependencies.clone(),
        debug: extract.debug,
        cull_input,
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
        cpu_reference_instances: context.virtual_geometry_cpu_reference_instances().to_vec(),
        bvh_visualization_instances,
        visible_cluster_ids,
        selected_clusters,
        selected_clusters_source:
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        node_and_cluster_cull_source: node_and_cluster_cull.source,
        node_and_cluster_cull_record_count: node_and_cluster_cull.record_count,
        node_and_cluster_cull_instance_seeds: node_and_cluster_cull.instance_seeds,
        node_and_cluster_cull_instance_work_items: node_and_cluster_cull.instance_work_items,
        node_and_cluster_cull_cluster_work_items: node_and_cluster_cull.cluster_work_items,
        node_and_cluster_cull_child_work_items: node_and_cluster_cull.child_work_items,
        node_and_cluster_cull_traversal_records: node_and_cluster_cull.traversal_records,
        node_and_cluster_cull_hierarchy_child_ids: extract.hierarchy_child_ids.clone(),
        node_and_cluster_cull_page_request_ids: node_and_cluster_cull.page_request_ids,
        node_and_cluster_cull_dispatch_setup: node_and_cluster_cull.dispatch_setup,
        node_and_cluster_cull_launch_worklist: node_and_cluster_cull.launch_worklist,
        node_and_cluster_cull_global_state: node_and_cluster_cull.global_state,
        hardware_rasterization_records: Vec::new(),
        hardware_rasterization_source:
            RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
        visbuffer_debug_marks,
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        visbuffer64_clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        visbuffer64_entries,
        requested_pages: page_upload_plan.requested_pages,
        resident_pages: page_upload_plan.resident_pages,
        dirty_requested_pages: page_upload_plan.dirty_requested_pages,
        evictable_pages: page_upload_plan.evictable_pages,
        resident_page_inspections,
        pending_page_request_inspections,
        available_page_slots,
        evictable_page_inspections,
        leaf_clusters,
        execution_segment_count: saturated_u32_len(execution.segments.len()),
        execution_page_count: saturated_u32_len(execution.page_ids.len()),
        execution_resident_segment_count: saturated_u32_len(execution.resident_segment_count),
        execution_pending_segment_count: saturated_u32_len(execution.pending_segment_count),
        execution_missing_segment_count: saturated_u32_len(execution.missing_segment_count),
        execution_repeated_draw_count: saturated_u32_len(execution.repeated_draw_count),
        execution_indirect_offsets: execution.indirect_offsets,
        execution_segments: execution.segments,
        submission_order: execution.submission_order,
        submission_records: execution.submission_records,
    })
}

fn build_cull_input_snapshot(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    available_page_slot_count: usize,
    evictable_page_count: usize,
) -> RenderVirtualGeometryCullInputSnapshot {
    RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: extract.cluster_budget,
        page_budget: extract.page_budget,
        instance_count: saturated_u32_len(extract.instances.len()),
        cluster_count: saturated_u32_len(extract.clusters.len()),
        page_count: saturated_u32_len(extract.pages.len()),
        visible_entity_count: unique_extract_entity_count(extract),
        visible_cluster_count: saturated_u32_len(extract.clusters.len()),
        resident_page_count: saturated_u32_len(page_upload_plan.resident_pages.len()),
        pending_page_request_count: saturated_u32_len(page_upload_plan.requested_pages.len()),
        available_page_slot_count: saturated_u32_len(available_page_slot_count),
        evictable_page_count: saturated_u32_len(evictable_page_count),
        debug: extract.debug,
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
    }
}

fn unique_extract_entity_count(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
) -> u32 {
    if !extract.instances.is_empty() {
        return saturated_u32_len(
            extract
                .instances
                .iter()
                .map(|instance| instance.entity)
                .collect::<BTreeSet<_>>()
                .len(),
        );
    }

    saturated_u32_len(
        extract
            .clusters
            .iter()
            .map(|cluster| cluster.entity)
            .collect::<BTreeSet<_>>()
            .len(),
    )
}

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn build_resident_page_inspections(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    page_upload_plan
        .resident_pages
        .iter()
        .enumerate()
        .map(
            |(slot, page_id)| RenderVirtualGeometryResidentPageInspection {
                page_id: *page_id,
                slot: u32::try_from(slot).unwrap_or(u32::MAX),
                size_bytes: page_size_bytes(extract, *page_id),
            },
        )
        .collect()
}

fn build_available_page_slots(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
) -> Vec<u32> {
    let resident_slot_count = page_upload_plan.resident_pages.len() as u32;
    (resident_slot_count..extract.page_budget)
        .take(page_upload_plan.requested_pages.len())
        .collect()
}

fn build_pending_page_request_inspections(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    context: &FrameSubmissionContext,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    available_page_slots: &[u32],
) -> Vec<RenderVirtualGeometryPageRequestInspection> {
    page_upload_plan
        .requested_pages
        .iter()
        .enumerate()
        .map(
            |(frontier_rank, page_id)| RenderVirtualGeometryPageRequestInspection {
                page_id: *page_id,
                size_bytes: page_size_bytes(extract, *page_id),
                generation: context.predicted_generation(),
                frontier_rank: u32::try_from(frontier_rank).unwrap_or(u32::MAX),
                assigned_slot: available_page_slots.get(frontier_rank).copied(),
                recycled_page_id: None,
            },
        )
        .collect()
}

fn build_evictable_page_inspections(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    resident_page_inspections: &[RenderVirtualGeometryResidentPageInspection],
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    page_upload_plan
        .evictable_pages
        .iter()
        .map(|page_id| {
            resident_page_inspections
                .iter()
                .find(|inspection| inspection.page_id == *page_id)
                .cloned()
                .unwrap_or(RenderVirtualGeometryResidentPageInspection {
                    page_id: *page_id,
                    slot: u32::MAX,
                    size_bytes: page_size_bytes(extract, *page_id),
                })
        })
        .collect()
}

fn page_size_bytes(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    page_id: u32,
) -> u64 {
    extract
        .pages
        .iter()
        .find(|page| page.page_id == page_id)
        .map(|page| page.size_bytes)
        .unwrap_or(0)
}

struct NodeAndClusterCullSnapshot {
    source: RenderVirtualGeometryNodeAndClusterCullSource,
    record_count: u32,
    global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    launch_worklist: Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    cluster_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullClusterWorkItem>,
    child_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullChildWorkItem>,
    traversal_records: Vec<RenderVirtualGeometryNodeAndClusterCullTraversalRecord>,
    page_request_ids: Vec<u32>,
}

fn build_node_and_cluster_cull_snapshot(
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
    let camera = &frame_extract.view.camera;
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
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
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
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
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
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
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
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
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
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
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

struct ExecutionSnapshot {
    page_ids: BTreeSet<u32>,
    resident_segment_count: usize,
    pending_segment_count: usize,
    missing_segment_count: usize,
    repeated_draw_count: usize,
    indirect_offsets: Vec<u64>,
    segments: Vec<RenderVirtualGeometryExecutionSegment>,
    submission_order: Vec<RenderVirtualGeometrySubmissionEntry>,
    submission_records: Vec<RenderVirtualGeometrySubmissionRecord>,
}

fn build_execution_snapshot(
    draw_segments: &[VisibilityVirtualGeometryDrawSegment],
    resident_page_set: &BTreeSet<u32>,
    requested_page_set: &BTreeSet<u32>,
) -> ExecutionSnapshot {
    let mut seen_pages = BTreeSet::new();
    let mut page_ids = BTreeSet::new();
    let mut resident_segment_count = 0;
    let mut pending_segment_count = 0;
    let mut missing_segment_count = 0;
    let mut repeated_draw_count = 0;
    let mut indirect_offsets = Vec::with_capacity(draw_segments.len());
    let mut segments = Vec::with_capacity(draw_segments.len());
    let mut submission_order = Vec::with_capacity(draw_segments.len());
    let mut submission_records = Vec::with_capacity(draw_segments.len());

    for (index, segment) in draw_segments.iter().enumerate() {
        let state =
            execution_state_for_page(segment.page_id, resident_page_set, requested_page_set);
        match state {
            RenderVirtualGeometryExecutionState::Resident => resident_segment_count += 1,
            RenderVirtualGeometryExecutionState::PendingUpload => pending_segment_count += 1,
            RenderVirtualGeometryExecutionState::Missing => missing_segment_count += 1,
        }
        if !seen_pages.insert(segment.page_id) {
            repeated_draw_count += 1;
        }
        page_ids.insert(segment.page_id);

        let index_u32 = u32::try_from(index).unwrap_or(u32::MAX);
        let instance_index = Some(0);
        indirect_offsets.push(index as u64);
        segments.push(RenderVirtualGeometryExecutionSegment {
            original_index: index_u32,
            instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
            draw_ref_index: index_u32,
            submission_index: Some(index_u32),
            draw_ref_rank: Some(index_u32),
            cluster_start_ordinal: segment.cluster_ordinal,
            cluster_span_count: segment.cluster_span_count,
            cluster_total_count: segment.cluster_count,
            submission_slot: Some(index_u32),
            state,
            lineage_depth: segment.lineage_depth,
            lod_level: segment.lod_level,
            frontier_rank: index_u32,
        });
        submission_order.push(RenderVirtualGeometrySubmissionEntry {
            instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
        });
        submission_records.push(RenderVirtualGeometrySubmissionRecord {
            instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
            draw_ref_index: Some(index_u32),
            submission_index: index_u32,
            draw_ref_rank: index_u32,
            original_index: index_u32,
        });
    }

    ExecutionSnapshot {
        page_ids,
        resident_segment_count,
        pending_segment_count,
        missing_segment_count,
        repeated_draw_count,
        indirect_offsets,
        segments,
        submission_order,
        submission_records,
    }
}

fn execution_state_for_page(
    page_id: u32,
    resident_page_set: &BTreeSet<u32>,
    requested_page_set: &BTreeSet<u32>,
) -> RenderVirtualGeometryExecutionState {
    if resident_page_set.contains(&page_id) {
        RenderVirtualGeometryExecutionState::Resident
    } else if requested_page_set.contains(&page_id) {
        RenderVirtualGeometryExecutionState::PendingUpload
    } else {
        RenderVirtualGeometryExecutionState::Missing
    }
}

fn instance_index_for_cluster_array_index(
    instances: &[crate::core::framework::render::RenderVirtualGeometryInstance],
    cluster_array_index: usize,
) -> Option<u32> {
    let cluster_array_index = u32::try_from(cluster_array_index).ok()?;
    instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            cluster_array_index >= instance.cluster_offset
                && cluster_array_index
                    < instance
                        .cluster_offset
                        .saturating_add(instance.cluster_count)
        })
        .and_then(|(instance_index, _)| u32::try_from(instance_index).ok())
}

fn visbuffer_mark_color(cluster_id: u32, page_id: u32, lod_level: u8) -> [u8; 4] {
    let lod_level = u32::from(lod_level);
    [
        (32 + ((cluster_id * 17 + page_id * 13) % 192)) as u8,
        (32 + ((page_id * 11 + lod_level * 7) % 192)) as u8,
        (32 + ((cluster_id * 5 + lod_level * 19) % 192)) as u8,
        255,
    ]
}

fn build_selected_clusters_from_visibility_feedback(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    visible_cluster_id_set: &BTreeSet<u32>,
    resident_page_set: &BTreeSet<u32>,
    requested_page_set: &BTreeSet<u32>,
) -> Vec<RenderVirtualGeometrySelectedCluster> {
    extract
        .clusters
        .iter()
        .enumerate()
        .filter(|(_, cluster)| visible_cluster_id_set.contains(&cluster.cluster_id))
        .map(|(cluster_array_index, cluster)| RenderVirtualGeometrySelectedCluster {
            instance_index: instance_index_for_cluster_array_index(
                &extract.instances,
                cluster_array_index,
            ),
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            cluster_ordinal: cluster_ordinal_for_entity(extract, cluster),
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            state: if resident_page_set.contains(&cluster.page_id) {
                crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident
            } else if requested_page_set.contains(&cluster.page_id) {
                crate::core::framework::render::RenderVirtualGeometryExecutionState::PendingUpload
            } else {
                crate::core::framework::render::RenderVirtualGeometryExecutionState::Missing
            },
        })
        .collect()
}

fn cluster_ordinal_for_entity(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    cluster: &crate::core::framework::render::RenderVirtualGeometryCluster,
) -> u32 {
    let mut cluster_ids = if extract.instances.is_empty() {
        extract
            .clusters
            .iter()
            .filter(|candidate| candidate.entity == cluster.entity)
            .map(|candidate| candidate.cluster_id)
            .collect::<Vec<_>>()
    } else {
        extract
            .instances
            .iter()
            .filter(|instance| instance.entity == cluster.entity)
            .flat_map(|instance| {
                let start = instance.cluster_offset as usize;
                let end = start.saturating_add(instance.cluster_count as usize);
                extract
                    .clusters
                    .get(start..end)
                    .into_iter()
                    .flatten()
                    .map(|candidate| candidate.cluster_id)
            })
            .collect::<Vec<_>>()
    };

    cluster_ids.sort_unstable();
    cluster_ids.dedup();
    cluster_ids
        .iter()
        .position(|cluster_id| cluster_id == cluster.cluster_id)
        .unwrap_or_default() as u32
}

fn build_visbuffer_debug_marks_from_selected_clusters(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBufferMark> {
    selected_clusters
        .iter()
        .map(|cluster| RenderVirtualGeometryVisBufferMark {
            instance_index: cluster.instance_index,
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            state: cluster.state,
            color_rgba: visbuffer_mark_color(
                cluster.cluster_id,
                cluster.page_id,
                cluster.lod_level,
            ),
        })
        .collect()
}

fn build_visbuffer64_entries_from_selected_clusters(
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    selected_clusters
        .iter()
        .enumerate()
        .map(|(entry_index, cluster)| {
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                u32::try_from(entry_index).unwrap_or(u32::MAX),
                cluster,
            )
        })
        .collect()
}
