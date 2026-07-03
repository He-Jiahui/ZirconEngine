mod execution;
mod node_cull;
mod page;
mod support;

use std::collections::BTreeSet;

use self::execution::{
    build_execution_snapshot, build_hardware_rasterization_records_from_execution_segments,
    build_selected_clusters_from_execution_segments,
    build_visbuffer64_entries_from_selected_clusters,
    build_visbuffer_debug_marks_from_selected_clusters,
    hardware_rasterization_source_for_execution, selected_cluster_source_for_execution,
    visbuffer64_source_for_execution,
};
use self::node_cull::build_node_and_cluster_cull_snapshot;
use self::page::{
    build_available_page_slots, build_cull_input_snapshot, build_evictable_page_inspections,
    build_pending_page_request_inspections, build_resident_page_inspections,
};
use self::support::saturated_u32_len;
use super::super::frame_submission_context::FrameSubmissionContext;
use crate::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryVisBuffer64Entry,
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
    let cull_input = build_cull_input_snapshot(
        extract,
        &page_upload_plan,
        available_page_slots.len(),
        evictable_page_inspections.len(),
    );
    let node_and_cluster_cull =
        build_node_and_cluster_cull_snapshot(frame_extract, context, cull_input);
    let draw_segments = context
        .visibility_context()
        .virtual_geometry_draw_segments
        .as_slice();
    let execution = build_execution_snapshot(
        extract,
        draw_segments,
        &resident_page_set,
        &requested_page_set,
    );
    let selected_clusters =
        build_selected_clusters_from_execution_segments(extract, &execution.segments);
    let visbuffer_debug_marks = extract
        .debug
        .visualize_visbuffer
        .then(|| build_visbuffer_debug_marks_from_selected_clusters(&selected_clusters))
        .unwrap_or_default();
    let hardware_rasterization_records =
        build_hardware_rasterization_records_from_execution_segments(draw_segments, &execution);
    let render_path_has_execution_selections = !execution.segments.is_empty();
    let selected_clusters_source =
        selected_cluster_source_for_execution(render_path_has_execution_selections);
    let hardware_rasterization_source =
        hardware_rasterization_source_for_execution(render_path_has_execution_selections);
    let visbuffer64_source = visbuffer64_source_for_execution(render_path_has_execution_selections);
    let visbuffer64_entries = build_visbuffer64_entries_from_selected_clusters(&selected_clusters);

    Some(RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        page_dependencies: extract.page_dependencies.clone(),
        resident_page_payloads: context.virtual_geometry_resident_page_payloads().to_vec(),
        debug: extract.debug,
        cull_input,
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
        cpu_reference_instances: context.virtual_geometry_cpu_reference_instances().to_vec(),
        bvh_visualization_instances,
        visible_cluster_ids,
        selected_clusters,
        selected_clusters_source,
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
        hardware_rasterization_records,
        hardware_rasterization_source,
        visbuffer_debug_marks,
        visbuffer64_source,
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
