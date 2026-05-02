use std::collections::BTreeSet;

use super::super::frame_submission_context::FrameSubmissionContext;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
};

pub(super) fn build_virtual_geometry_debug_snapshot(
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
    let resident_page_inspections = Vec::<RenderVirtualGeometryResidentPageInspection>::new();
    let pending_page_request_inspections = Vec::<RenderVirtualGeometryPageRequestInspection>::new();
    let available_page_slots = Vec::new();
    let evictable_page_inspections = Vec::<RenderVirtualGeometryResidentPageInspection>::new();

    Some(RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        debug: extract.debug,
        cull_input: build_cull_input_snapshot(extract),
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        cpu_reference_instances: context.virtual_geometry_cpu_reference_instances().to_vec(),
        bvh_visualization_instances,
        visible_cluster_ids,
        selected_clusters,
        selected_clusters_source: RenderVirtualGeometrySelectedClusterSource::Unavailable,
        node_and_cluster_cull_source: RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
        node_and_cluster_cull_record_count: 0,
        node_and_cluster_cull_instance_seeds: Vec::<
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
        >::new(),
        node_and_cluster_cull_instance_work_items: Vec::new(),
        node_and_cluster_cull_cluster_work_items: Vec::new(),
        node_and_cluster_cull_child_work_items: Vec::new(),
        node_and_cluster_cull_traversal_records: Vec::new(),
        node_and_cluster_cull_hierarchy_child_ids: Vec::new(),
        node_and_cluster_cull_page_request_ids: Vec::new(),
        node_and_cluster_cull_dispatch_setup: None,
        node_and_cluster_cull_launch_worklist: None,
        node_and_cluster_cull_global_state: None,
        hardware_rasterization_records: Vec::new(),
        hardware_rasterization_source:
            RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
        visbuffer_debug_marks,
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::Unavailable,
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
        execution_segment_count: 0,
        execution_page_count: 0,
        execution_resident_segment_count: 0,
        execution_pending_segment_count: 0,
        execution_missing_segment_count: 0,
        execution_repeated_draw_count: 0,
        execution_indirect_offsets: Vec::new(),
        execution_segments: Vec::new(),
        submission_order: Vec::new(),
        submission_records: Vec::new(),
    })
}

fn build_cull_input_snapshot(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
) -> RenderVirtualGeometryCullInputSnapshot {
    RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: extract.cluster_budget,
        page_budget: extract.page_budget,
        instance_count: saturated_u32_len(extract.instances.len()),
        cluster_count: saturated_u32_len(extract.clusters.len()),
        page_count: saturated_u32_len(extract.pages.len()),
        visible_entity_count: unique_extract_entity_count(extract),
        visible_cluster_count: saturated_u32_len(extract.clusters.len()),
        resident_page_count: 0,
        pending_page_request_count: 0,
        available_page_slot_count: 0,
        evictable_page_count: 0,
        debug: extract.debug,
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
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

fn instance_index_for_cluster_ordinal(
    instances: &[crate::core::framework::render::RenderVirtualGeometryInstance],
    cluster_ordinal: usize,
) -> Option<u32> {
    let cluster_ordinal = u32::try_from(cluster_ordinal).ok()?;
    instances
        .iter()
        .enumerate()
        .find(|(_, instance)| {
            cluster_ordinal >= instance.cluster_offset
                && cluster_ordinal
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
        .map(|(cluster_ordinal, cluster)| RenderVirtualGeometrySelectedCluster {
            instance_index: instance_index_for_cluster_ordinal(
                &extract.instances,
                cluster_ordinal,
            ),
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            cluster_ordinal: u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
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
