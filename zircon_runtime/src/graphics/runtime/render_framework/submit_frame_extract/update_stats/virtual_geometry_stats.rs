use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};
use std::collections::BTreeSet;

pub(super) fn update_virtual_geometry_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
) {
    let virtual_geometry_stats = record_update.virtual_geometry_stats();
    let virtual_geometry_extract = context.virtual_geometry_extract();
    state.stats.last_virtual_geometry_cluster_budget = virtual_geometry_extract
        .map(|extract| extract.cluster_budget as usize)
        .unwrap_or(0);
    state.stats.last_virtual_geometry_page_budget = virtual_geometry_extract
        .map(|extract| extract.page_budget as usize)
        .unwrap_or(0);
    state.stats.last_virtual_geometry_input_cluster_count = virtual_geometry_extract
        .map(|extract| extract.clusters.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_input_page_count = virtual_geometry_extract
        .map(|extract| extract.pages.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_visible_cluster_count = context
        .visibility_context()
        .virtual_geometry_visible_clusters
        .len();
    state.stats.last_virtual_geometry_visible_entity_count = virtual_geometry_extract
        .map(visible_entity_count_from_extract)
        .unwrap_or(0);
    state.stats.last_virtual_geometry_instance_count = virtual_geometry_extract
        .map(|extract| extract.instances.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_requested_page_count = context
        .virtual_geometry_page_upload_plan()
        .map(|plan| plan.requested_pages.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_dirty_page_count = context
        .virtual_geometry_page_upload_plan()
        .map(|plan| plan.dirty_requested_pages.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_forced_mip =
        virtual_geometry_extract.and_then(|extract| extract.debug.forced_mip);
    state.stats.last_virtual_geometry_freeze_cull = virtual_geometry_extract
        .map(|extract| extract.debug.freeze_cull)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_visualize_bvh = virtual_geometry_extract
        .map(|extract| extract.debug.visualize_bvh)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_visualize_visbuffer = virtual_geometry_extract
        .map(|extract| extract.debug.visualize_visbuffer)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_print_leaf_clusters = virtual_geometry_extract
        .map(|extract| extract.debug.print_leaf_clusters)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_page_table_entry_count =
        virtual_geometry_stats.page_table_entry_count();
    state.stats.last_virtual_geometry_resident_page_count =
        virtual_geometry_stats.resident_page_count();
    state.stats.last_virtual_geometry_pending_request_count =
        virtual_geometry_stats.pending_request_count();
    state.stats.last_virtual_geometry_page_dependency_count =
        virtual_geometry_stats.page_dependency_count();
    state.stats.last_virtual_geometry_completed_page_count =
        virtual_geometry_stats.completed_page_count();
    state.stats.last_virtual_geometry_replaced_page_count =
        virtual_geometry_stats.replaced_page_count();
    let execution_stats = virtual_geometry_execution_stats(context);
    state.stats.last_virtual_geometry_indirect_draw_count = execution_stats.segment_count;
    state.stats.last_virtual_geometry_indirect_buffer_count = 0;
    state.stats.last_virtual_geometry_indirect_args_count = execution_stats.segment_count;
    state.stats.last_virtual_geometry_indirect_segment_count =
        virtual_geometry_stats.indirect_segment_count();
    state.stats.last_virtual_geometry_execution_segment_count = execution_stats.segment_count;
    state.stats.last_virtual_geometry_execution_page_count = execution_stats.page_count;
    state
        .stats
        .last_virtual_geometry_execution_resident_segment_count =
        execution_stats.resident_segment_count;
    state
        .stats
        .last_virtual_geometry_execution_pending_segment_count =
        execution_stats.pending_segment_count;
    state
        .stats
        .last_virtual_geometry_execution_missing_segment_count =
        execution_stats.missing_segment_count;
    state
        .stats
        .last_virtual_geometry_execution_repeated_draw_count = execution_stats.repeated_draw_count;
    state
        .stats
        .last_virtual_geometry_cluster_selection_input_source = virtual_geometry_extract
        .map(|_| RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned)
        .unwrap_or_default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_source = virtual_geometry_extract
        .filter(|extract| !extract.instances.is_empty())
        .map(|_| RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput)
        .unwrap_or_default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_record_count =
        if virtual_geometry_extract.is_some_and(|extract| !extract.instances.is_empty()) {
            1
        } else {
            0
        };
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_dispatch_group_count =
        virtual_geometry_extract
            .map(|extract| [extract.instances.len().max(1).div_ceil(64), 1, 1])
            .unwrap_or([0, 0, 0]);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_seed_count = virtual_geometry_extract
        .map(|extract| extract.instances.len())
        .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_work_item_count =
        virtual_geometry_extract
            .map(|extract| extract.instances.len())
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count =
        virtual_geometry_extract
            .map(|extract| {
                extract
                    .instances
                    .iter()
                    .map(|instance| instance.cluster_count as usize)
                    .sum()
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count =
        virtual_geometry_extract
            .map(|extract| extract.hierarchy_child_ids.len())
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_child_work_item_count =
        virtual_geometry_extract
            .map(node_and_cluster_cull_traversal_stats)
            .map(|stats| stats.child_work_item_count)
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_traversal_record_count =
        virtual_geometry_extract
            .map(node_and_cluster_cull_traversal_stats)
            .map(|stats| stats.traversal_record_count)
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_page_request_count = 0;
    state.stats.last_virtual_geometry_selected_cluster_source = virtual_geometry_extract
        .map(|_| RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections)
        .unwrap_or_default();
    state.stats.last_virtual_geometry_selected_cluster_count = context
        .visibility_context()
        .virtual_geometry_visible_clusters
        .len();
    state.stats.last_virtual_geometry_visbuffer64_source = virtual_geometry_extract
        .map(|_| RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections)
        .unwrap_or_default();
    state.stats.last_virtual_geometry_visbuffer64_entry_count = context
        .visibility_context()
        .virtual_geometry_visible_clusters
        .len();
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_source =
        RenderVirtualGeometryHardwareRasterizationSource::Unavailable;
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_record_count = 0;
}

pub(super) fn reset_virtual_geometry_stats(state: &mut RenderFrameworkState) {
    state.stats.last_virtual_geometry_cluster_budget = 0;
    state.stats.last_virtual_geometry_page_budget = 0;
    state.stats.last_virtual_geometry_input_cluster_count = 0;
    state.stats.last_virtual_geometry_input_page_count = 0;
    state.stats.last_virtual_geometry_visible_cluster_count = 0;
    state.stats.last_virtual_geometry_visible_entity_count = 0;
    state.stats.last_virtual_geometry_instance_count = 0;
    state.stats.last_virtual_geometry_requested_page_count = 0;
    state.stats.last_virtual_geometry_dirty_page_count = 0;
    state.stats.last_virtual_geometry_forced_mip = None;
    state.stats.last_virtual_geometry_freeze_cull = false;
    state.stats.last_virtual_geometry_visualize_bvh = false;
    state.stats.last_virtual_geometry_visualize_visbuffer = false;
    state.stats.last_virtual_geometry_print_leaf_clusters = false;
    state.stats.last_virtual_geometry_page_table_entry_count = 0;
    state.stats.last_virtual_geometry_resident_page_count = 0;
    state.stats.last_virtual_geometry_pending_request_count = 0;
    state.stats.last_virtual_geometry_page_dependency_count = 0;
    state.stats.last_virtual_geometry_completed_page_count = 0;
    state.stats.last_virtual_geometry_replaced_page_count = 0;
    state.stats.last_virtual_geometry_indirect_draw_count = 0;
    state.stats.last_virtual_geometry_indirect_buffer_count = 0;
    state.stats.last_virtual_geometry_indirect_args_count = 0;
    state.stats.last_virtual_geometry_indirect_segment_count = 0;
    state.stats.last_virtual_geometry_execution_segment_count = 0;
    state.stats.last_virtual_geometry_execution_page_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_resident_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_pending_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_missing_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_repeated_draw_count = 0;
    state
        .stats
        .last_virtual_geometry_cluster_selection_input_source = Default::default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_source = Default::default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_record_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_dispatch_group_count = [0, 0, 0];
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_seed_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_child_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_traversal_record_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_page_request_count = 0;
    state.stats.last_virtual_geometry_selected_cluster_source = Default::default();
    state.stats.last_virtual_geometry_selected_cluster_count = 0;
    state.stats.last_virtual_geometry_visbuffer64_source = Default::default();
    state.stats.last_virtual_geometry_visbuffer64_entry_count = 0;
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_source = Default::default();
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_record_count = 0;
}

fn visible_entity_count_from_extract(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
) -> usize {
    if !extract.instances.is_empty() {
        return extract
            .instances
            .iter()
            .map(|instance| instance.entity)
            .collect::<BTreeSet<_>>()
            .len();
    }

    extract
        .clusters
        .iter()
        .map(|cluster| cluster.entity)
        .collect::<BTreeSet<_>>()
        .len()
}

#[derive(Clone, Copy, Debug, Default)]
struct VirtualGeometryExecutionStats {
    segment_count: usize,
    page_count: usize,
    resident_segment_count: usize,
    pending_segment_count: usize,
    missing_segment_count: usize,
    repeated_draw_count: usize,
}

fn virtual_geometry_execution_stats(
    context: &FrameSubmissionContext,
) -> VirtualGeometryExecutionStats {
    let Some(page_upload_plan) = context.virtual_geometry_page_upload_plan() else {
        return VirtualGeometryExecutionStats::default();
    };
    let resident_pages = page_upload_plan
        .resident_pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let requested_pages = page_upload_plan
        .requested_pages
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut seen_pages = BTreeSet::new();
    let mut all_pages = BTreeSet::new();
    let mut stats = VirtualGeometryExecutionStats::default();

    for segment in &context.visibility_context().virtual_geometry_draw_segments {
        stats.segment_count += 1;
        if !seen_pages.insert(segment.page_id) {
            stats.repeated_draw_count += 1;
        }
        all_pages.insert(segment.page_id);
        match execution_state_for_page(segment.page_id, &resident_pages, &requested_pages) {
            RenderVirtualGeometryExecutionState::Resident => stats.resident_segment_count += 1,
            RenderVirtualGeometryExecutionState::PendingUpload => stats.pending_segment_count += 1,
            RenderVirtualGeometryExecutionState::Missing => stats.missing_segment_count += 1,
        }
    }

    stats.page_count = all_pages.len();
    stats
}

fn execution_state_for_page(
    page_id: u32,
    resident_pages: &BTreeSet<u32>,
    requested_pages: &BTreeSet<u32>,
) -> RenderVirtualGeometryExecutionState {
    if resident_pages.contains(&page_id) {
        RenderVirtualGeometryExecutionState::Resident
    } else if requested_pages.contains(&page_id) {
        RenderVirtualGeometryExecutionState::PendingUpload
    } else {
        RenderVirtualGeometryExecutionState::Missing
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct NodeAndClusterCullTraversalStats {
    child_work_item_count: usize,
    traversal_record_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct TraversalQueueItem {
    cluster_array_index: u32,
    hierarchy_node_id: Option<u32>,
}

fn node_and_cluster_cull_traversal_stats(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
) -> NodeAndClusterCullTraversalStats {
    let mut stats = NodeAndClusterCullTraversalStats::default();
    let mut queue = extract
        .instances
        .iter()
        .flat_map(|instance| {
            (instance.cluster_offset
                ..instance
                    .cluster_offset
                    .saturating_add(instance.cluster_count))
                .map(|cluster_array_index| TraversalQueueItem {
                    cluster_array_index,
                    hierarchy_node_id: extract
                        .clusters
                        .get(cluster_array_index as usize)
                        .and_then(|cluster| cluster.hierarchy_node_id),
                })
        })
        .collect::<Vec<_>>();
    let mut cursor = 0;

    while cursor < queue.len() {
        let item = queue[cursor];
        cursor += 1;
        stats.traversal_record_count += 1;

        let node = item
            .hierarchy_node_id
            .and_then(|node_id| hierarchy_node(extract, node_id));
        if let Some(node) = node.filter(|node| node.child_count > 0) {
            for child_table_index in
                node.child_base..node.child_base.saturating_add(node.child_count)
            {
                stats.traversal_record_count += 1;
                stats.child_work_item_count += 1;
                let child_node_id = extract
                    .hierarchy_child_ids
                    .get(child_table_index as usize)
                    .copied();
                queue.push(TraversalQueueItem {
                    cluster_array_index: item.cluster_array_index,
                    hierarchy_node_id: child_node_id,
                });
            }
            continue;
        }

        stats.traversal_record_count += 1;
    }

    stats
}

fn hierarchy_node(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    node_id: u32,
) -> Option<&crate::core::framework::render::RenderVirtualGeometryHierarchyNode> {
    extract
        .hierarchy_nodes
        .iter()
        .find(|node| node.node_id == node_id)
}
