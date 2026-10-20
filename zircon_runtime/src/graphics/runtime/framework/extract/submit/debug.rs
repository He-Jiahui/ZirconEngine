//! 虚拟几何调试产品汇集页、节点与执行快照，供编辑器观察同一相机帧的完整驻留决策。
#[path = "debug/execution.rs"]
mod execution;
#[path = "debug/node_cull.rs"]
mod node_cull;
#[path = "debug/page.rs"]
mod page;
#[path = "debug/support.rs"]
mod support;

use std::collections::BTreeSet;

use self::execution::{
    build_execution_snapshot, build_hardware_rasterization_records_from_execution_segments,
    build_selected_clusters_from_execution_segments,
    build_visbuffer64_entries_from_selected_clusters,
    build_visbuffer_debug_marks_from_selected_clusters,
    hardware_rasterization_source_for_execution, selected_cluster_source_for_execution,
    visbuffer64_source_for_execution, ExecutionLookup,
};
use self::node_cull::build_node_and_cluster_cull_snapshot;
use self::page::{
    build_available_page_slots, build_cull_input_snapshot, build_evictable_page_inspections,
    build_page_size_index, build_pending_page_request_inspections, build_resident_page_inspections,
};
use self::support::saturated_u32_len;
use super::super::frame_submission_context::FrameSubmissionContext;
use crate::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryVisBuffer64Entry,
};

// Public diagnostic payload sharing remains deferred. Count these optional
// copies; execution takes the originals only after diagnostic readers finish.
fn copy_resident_payloads(
    context: &FrameSubmissionContext,
) -> Vec<crate::core::framework::render::RenderVirtualGeometryPagePayload> {
    let payloads = context.virtual_geometry_resident_page_payloads();
    crate::profile_counter!(
        "render",
        "vg_payload_copy_bytes",
        payloads
            .iter()
            .map(|page| std::mem::size_of_val(page.vertices.as_slice())
                + std::mem::size_of_val(page.cluster_ranges.as_slice()))
            .sum::<usize>()
    );
    payloads.to_vec()
}

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
    let page_size_index = build_page_size_index(extract);
    let resident_page_inspections =
        build_resident_page_inspections(&page_upload_plan, &page_size_index);
    let available_page_slots = build_available_page_slots(extract, &page_upload_plan);
    let pending_page_request_inspections = build_pending_page_request_inspections(
        context,
        &page_upload_plan,
        &available_page_slots,
        &page_size_index,
    );
    let evictable_page_inspections = build_evictable_page_inspections(
        &page_upload_plan,
        &resident_page_inspections,
        &page_size_index,
    );
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
    let cull_input = build_virtual_geometry_cull_input(context)?;
    let node_and_cluster_cull =
        build_node_and_cluster_cull_snapshot(frame_extract, context, cull_input);
    let draw_segments = context
        .visibility_context()
        .virtual_geometry_draw_segments
        .as_slice();
    let execution_lookup = ExecutionLookup::new(extract);
    let execution = build_execution_snapshot(
        &execution_lookup,
        draw_segments,
        &resident_page_set,
        &requested_page_set,
    );
    let selected_clusters =
        build_selected_clusters_from_execution_segments(&execution_lookup, &execution.segments);
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
        resident_page_payloads: copy_resident_payloads(context),
        debug: extract.debug,
        cull_input,
        cluster_selection_input_source: cull_input.cluster_selection_input_source,
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

fn vg_execution_required<T>(value: Option<T>, reason: &str) -> Option<T> {
    #[cfg(feature = "native-host")]
    if value.is_none() && std::env::var_os("ZIRCON_VG_EXECUTION_DIAGNOSTICS").is_some() {
        eprintln!("VG_EXECUTION_UNAVAILABLE reason={reason}");
    }
    #[cfg(not(feature = "native-host"))]
    let _ = reason;
    value
}

pub(super) fn build_virtual_geometry_execution_data(
    context: &mut FrameSubmissionContext,
) -> Option<crate::core::framework::render::RenderVirtualGeometryExecutionData> {
    let extract = vg_execution_required(context.virtual_geometry_extract(), "extract")?;
    let plan = context
        .virtual_geometry_page_upload_plan()
        .cloned()
        .unwrap_or_default();
    let prepared = vg_execution_required(
        context.virtual_geometry_prepared_cpu_frame(),
        "prepared CPU frame",
    )?;
    let paired = &prepared.execution_inputs;
    let resident = match paired.prepare.as_ref() {
        Some(author) => author
            .resident_pages
            .iter()
            .chain(&author.evictable_pages)
            .map(|page| page.page_id)
            .collect::<BTreeSet<_>>(),
        None => plan.resident_pages.iter().copied().collect::<BTreeSet<_>>(),
    };
    let requested = match paired.prepare.as_ref() {
        Some(author) => author
            .pending_page_requests
            .iter()
            .map(|request| request.page_id)
            .collect::<BTreeSet<_>>(),
        None => plan
            .requested_pages
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
    };
    let lookup = ExecutionLookup::new(extract);
    let mut execution = build_execution_snapshot(
        &lookup,
        &context.visibility_context().virtual_geometry_draw_segments,
        &resident,
        &requested,
    );
    if paired.selections.is_some() || paired.prepare.is_some() {
        execution.segments = vg_execution_required(
            prepared.paired_execution_segments.clone(),
            "paired execution segments",
        )?;
    }
    // The main detail builder consumes these same segment values. Diagnostics
    // keep their historical projection; production gets the actual page owner.
    for segment in &mut execution.segments {
        let mut slot = None;
        for &(page, value) in &prepared.author_resident_page_slots {
            if page != segment.page_id {
                continue;
            }
            if slot.is_some_and(|previous| previous != value) {
                return vg_execution_required(None, "conflicting resident submission slots");
            }
            slot = Some(value);
        }
        let mut request = None;
        for fact in &prepared.author_pending_page_requests {
            if fact.0 != segment.page_id {
                continue;
            }
            if request.is_some_and(|previous| previous != *fact) {
                return vg_execution_required(None, "conflicting pending page requests");
            }
            request = Some(*fact);
        }
        if segment.state
            == crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident
        {
            segment.submission_slot =
                Some(vg_execution_required(slot, "resident submission slot")?);
        } else {
            segment.submission_slot = request.and_then(|fact| fact.2);
        }
        segment.frontier_rank = request.map(|fact| fact.1).unwrap_or(0);
    }
    // Full original author catalog. No visibility/refinement projection, debug
    // record, sorting or de-duplication participates in this copy.
    let author_cluster_catalog = extract.clusters.clone();
    let (selected_clusters, mut candidate_author_indices) = match paired.selections.as_ref() {
        Some(author) => vg_execution_required(
            paired_selection_catalog(extract, author),
            "paired selection catalog",
        )?,
        None => vg_execution_required(
            build_lossless_candidate_catalog(
                extract,
                &resident,
                &requested,
                &context.visibility_context().main_view_visible_entity_set(),
            ),
            "lossless candidate catalog",
        )?,
    };
    if paired.selections.is_none() && paired.prepare.is_some() {
        // A paired prepare may describe a slice of a larger declared entity.
        // Preserve that producer's original total; do not infer it from a
        // filtered catalog length, current rows or completion counts.
        for (candidate, provenance) in selected_clusters.iter().zip(&mut candidate_author_indices) {
            let mut total = None;
            for segment in &execution.segments {
                if segment.entity != candidate.entity
                    || segment.instance_index != candidate.instance_index
                    || segment.page_id != candidate.page_id
                {
                    continue;
                }
                let end = segment
                    .cluster_start_ordinal
                    .checked_add(segment.cluster_span_count)?;
                if candidate.cluster_ordinal < segment.cluster_start_ordinal
                    || candidate.cluster_ordinal >= end
                {
                    continue;
                }
                if total.is_some_and(|previous| previous != segment.cluster_total_count) {
                    return None;
                }
                total = Some(segment.cluster_total_count);
            }
            if let Some(value) = total {
                if candidate.cluster_ordinal >= value {
                    return None;
                }
                provenance.2 = value;
            }
        }
    }
    let cull_input =
        vg_execution_required(build_virtual_geometry_cull_input(context), "cull input")?;
    let sources = context.virtual_geometry_indexed_sources();
    let indexed_candidate_payload_ranges = if sources
        .iter()
        .any(|source| source.has_authored_execution_binding())
    {
        let mut bindings = Vec::new();
        let mut claimed = BTreeSet::new();
        for (candidate_index, (candidate, &(_, raw_index, _))) in selected_clusters
            .iter()
            .zip(&candidate_author_indices)
            .enumerate()
        {
            let key = match candidate.instance_index {
                Some(index) => extract
                    .instances
                    .get(index as usize)?
                    .stable_instance_key_or_legacy(),
                None => crate::core::framework::render::render_mesh_stable_instance_key(
                    candidate.entity,
                    0,
                ),
            };
            let matches = sources
                .iter()
                .flat_map(|source| source.authored_rows().unwrap_or(&[]))
                .filter(|binding| {
                    binding.instance_index == candidate.instance_index
                        && binding.stable_instance_key == key
                        && binding.author_array_index == raw_index
                })
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return vg_execution_required(None, "authored candidate binding");
            }
            for binding in matches {
                if !claimed.insert((
                    candidate_index,
                    binding.payload_index,
                    binding.payload_range_index,
                )) {
                    return vg_execution_required(None, "duplicate physical candidate binding");
                }
                bindings.push((
                    u32::try_from(candidate_index).ok()?,
                    binding.payload_index,
                    binding.payload_range_index,
                ));
            }
        }
        Some(bindings)
    } else {
        None
    };
    let instances = extract.instances.clone();
    let debug = extract.debug;
    // All validation and diagnostic reads precede ownership transfer. On failure,
    // leave the context intact so its caller can still report the original inputs.
    let resident_page_payloads = context.take_virtual_geometry_resident_page_payloads();
    Some(
        crate::core::framework::render::RenderVirtualGeometryExecutionData {
            indexed_candidate_payload_ranges,
            author_cluster_catalog,
            candidate_author_indices,
            instances,
            resident_page_payloads,
            debug,
            cull_input,
            prepared_cpu_frame: Some(prepared),
            selected_clusters,
            execution_segments: execution.segments,
        },
    )
}

pub(super) fn build_virtual_geometry_cull_input(
    context: &FrameSubmissionContext,
) -> Option<crate::core::framework::render::RenderVirtualGeometryCullInputSnapshot> {
    context.cached_virtual_geometry_cull_input(|| {
        let extract = context.virtual_geometry_extract()?;
        let plan = context
            .virtual_geometry_page_upload_plan()
            .cloned()
            .unwrap_or_default();
        let available = build_available_page_slots(extract, &plan);
        let sizes = build_page_size_index(extract);
        let pages = build_resident_page_inspections(&plan, &sizes);
        let evictable = build_evictable_page_inspections(&plan, &pages, &sizes);
        let mut cull_input = build_cull_input_snapshot(
            extract,
            &plan,
            available.len(),
            evictable.len(),
            &context.visibility_context().main_view_visible_entity_set(),
        )
        .ok()?;
        let inputs = context.virtual_geometry_execution_inputs();
        if let Some(author) = inputs.prepare.as_ref() {
            cull_input.visible_entity_count = u32::try_from(author.visible_entities.len()).ok()?;
            cull_input.visible_cluster_count = u32::try_from(author.visible_clusters.len()).ok()?;
            cull_input.resident_page_count = u32::try_from(author.resident_pages.len()).ok()?;
            cull_input.pending_page_request_count =
                u32::try_from(author.pending_page_requests.len()).ok()?;
            cull_input.available_page_slot_count =
                u32::try_from(author.available_slots.len()).ok()?;
            cull_input.evictable_page_count = u32::try_from(author.evictable_pages.len()).ok()?;
            cull_input.cluster_selection_input_source =
                RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned;
        }
        if inputs.selections.is_some() {
            cull_input.cluster_selection_input_source =
                RenderVirtualGeometryClusterSelectionInputSource::ExplicitFrameOwned;
        }
        Some(cull_input)
    })
}

impl FrameSubmissionContext {
    pub(in crate::graphics::runtime::render_framework) fn virtual_geometry_cull_input(
        &self,
    ) -> Option<crate::core::framework::render::RenderVirtualGeometryCullInputSnapshot> {
        build_virtual_geometry_cull_input(self)
    }
}

fn build_lossless_candidate_catalog(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    resident: &BTreeSet<u32>,
    requested: &BTreeSet<u32>,
    visible_legacy_entities: &BTreeSet<u64>,
) -> Option<(
    Vec<crate::core::framework::render::RenderVirtualGeometrySelectedCluster>,
    Vec<(u32, u32, u32)>,
)> {
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometrySelectedCluster,
    };
    let active = extract
        .checked_active_cluster_rows(visible_legacy_entities)
        .ok()?;
    let mut candidates = Vec::new();
    let mut provenance = Vec::new();
    for (instance_index, instance) in extract.instances.iter().enumerate() {
        let instance_index = u32::try_from(instance_index).ok()?;
        extract
            .cluster_row_indices(
                instance.entity,
                instance.stable_instance_key_or_legacy(),
                Some(instance_index),
            )
            .ok()?;
        let ids = crate::graphics::visibility::cluster_ids_for_stable_instance_key(
            extract,
            instance.stable_instance_key_or_legacy(),
        );
        let total = u32::try_from(ids.len().max(1)).ok()?;
        let start = usize::try_from(instance.cluster_offset).ok()?;
        let end = usize::try_from(
            instance
                .cluster_offset
                .checked_add(instance.cluster_count)?,
        )
        .ok()?;
        for (local_index, cluster) in extract.clusters.get(start..end)?.iter().enumerate() {
            if cluster.entity != instance.entity {
                return None;
            }
            let array_index = start.checked_add(local_index)?;
            let ordinal = u32::try_from(ids.binary_search(&cluster.cluster_id).ok()?).ok()?;
            let state = if resident.contains(&cluster.page_id) {
                RenderVirtualGeometryExecutionState::Resident
            } else if requested.contains(&cluster.page_id) {
                RenderVirtualGeometryExecutionState::PendingUpload
            } else {
                RenderVirtualGeometryExecutionState::Missing
            };
            candidates.push(RenderVirtualGeometrySelectedCluster {
                instance_index: Some(instance_index),
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: ordinal,
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                state,
            });
            provenance.push((instance_index, u32::try_from(array_index).ok()?, total));
        }
    }
    {
        // Instance-less authored input remains instance-less. The policy key is
        // the existing legacy key; raw array index and policy ordinal differ.
        for (array_index, cluster) in extract.clusters.iter().enumerate() {
            if !active
                .iter()
                .any(|&(instance, _, row)| instance.is_none() && row == array_index)
            {
                continue;
            }
            let key =
                crate::core::framework::render::render_mesh_stable_instance_key(cluster.entity, 0);
            let ids =
                crate::graphics::visibility::cluster_ids_for_stable_instance_key(extract, key);
            let ordinal = u32::try_from(ids.binary_search(&cluster.cluster_id).ok()?).ok()?;
            let total = u32::try_from(ids.len().max(1)).ok()?;
            let state = if resident.contains(&cluster.page_id) {
                RenderVirtualGeometryExecutionState::Resident
            } else if requested.contains(&cluster.page_id) {
                RenderVirtualGeometryExecutionState::PendingUpload
            } else {
                RenderVirtualGeometryExecutionState::Missing
            };
            candidates.push(RenderVirtualGeometrySelectedCluster {
                instance_index: None,
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: ordinal,
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                state,
            });
            provenance.push((u32::MAX, u32::try_from(array_index).ok()?, total));
        }
    }
    Some((candidates, provenance))
}

#[cfg(test)]
mod lossless_indexed_author_tests {
    use super::*;
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryExecutionState,
        RenderVirtualGeometryExtract, RenderVirtualGeometryInstance,
    };
    #[test]
    fn normal_catalog_keeps_raw_duplicates_missing_rows_and_separate_policy_ordinals() {
        let extract = RenderVirtualGeometryExtract {
            instances: vec![RenderVirtualGeometryInstance {
                entity: 7,
                stable_instance_key: 77,
                cluster_offset: 0,
                cluster_count: 3,
                ..Default::default()
            }],
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity: 7,
                    cluster_id: 9,
                    page_id: 30,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 7,
                    cluster_id: 2,
                    page_id: 40,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 7,
                    cluster_id: 9,
                    page_id: 30,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let (rows, provenance) = build_lossless_candidate_catalog(
            &extract,
            &BTreeSet::from([30]),
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect("valid original ranges");
        assert_eq!(
            rows.iter()
                .map(|row| (row.cluster_id, row.cluster_ordinal))
                .collect::<Vec<_>>(),
            vec![(9, 1), (2, 0), (9, 1)]
        );
        assert_eq!(provenance, vec![(0, 0, 2), (0, 1, 2), (0, 2, 2)]);
        assert_eq!(
            rows.iter().map(|row| row.state).collect::<Vec<_>>(),
            vec![
                RenderVirtualGeometryExecutionState::Resident,
                RenderVirtualGeometryExecutionState::Missing,
                RenderVirtualGeometryExecutionState::Resident,
            ]
        );
        assert_eq!(extract.clusters.len(), 3);
    }
}

fn paired_selection_catalog(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
    selections: &[crate::core::framework::render::RenderVirtualGeometryClusterSelectionInput],
) -> Option<(
    Vec<crate::core::framework::render::RenderVirtualGeometrySelectedCluster>,
    Vec<(u32, u32, u32)>,
)> {
    use crate::core::framework::render::RenderVirtualGeometrySelectedCluster;
    let mut rows = Vec::with_capacity(selections.len());
    let mut provenance = Vec::with_capacity(selections.len());
    let mut occurrences =
        std::collections::BTreeMap::<(Option<u32>, u64, u32, u32, u8), usize>::new();
    for author in selections {
        let key = match author.instance_index {
            Some(index) => extract
                .instances
                .get(usize::try_from(index).ok()?)?
                .stable_instance_key_or_legacy(),
            None => {
                crate::core::framework::render::render_mesh_stable_instance_key(author.entity, 0)
            }
        };
        let domain = extract
            .cluster_row_indices(author.entity, key, author.instance_index)
            .ok()?;
        let mut matches = Vec::new();
        for index in domain {
            let cluster = extract.clusters.get(index)?;
            if cluster.entity == author.entity
                && cluster.cluster_id == author.cluster_id
                && cluster.page_id == author.page_id
                && cluster.lod_level == author.lod_level
            {
                matches.push(u32::try_from(index).ok()?);
            }
        }
        if matches.is_empty() {
            return None;
        }
        let key = (
            author.instance_index,
            author.entity,
            author.cluster_id,
            author.page_id,
            author.lod_level,
        );
        let occurrence = occurrences.entry(key).or_insert(0);
        // A unique raw source may be intentionally admitted multiple times. For
        // repeated raw sources preserve their original occurrence order.
        let raw = if matches.len() == 1 {
            matches[0]
        } else {
            *matches.get(*occurrence)?
        };
        *occurrence = occurrence.checked_add(1)?;
        let total = u32::try_from(author.entity_cluster_total_count).ok()?;
        if author.cluster_ordinal >= total {
            return None;
        }
        rows.push(RenderVirtualGeometrySelectedCluster {
            instance_index: author.instance_index,
            entity: author.entity,
            cluster_id: author.cluster_id,
            cluster_ordinal: author.cluster_ordinal,
            page_id: author.page_id,
            lod_level: author.lod_level,
            state: author.state,
        });
        provenance.push((author.instance_index.unwrap_or(u32::MAX), raw, total));
    }
    Some((rows, provenance))
}

#[cfg(test)]
mod paired_indexed_author_contract_tests {
    use super::*;
    use crate::core::framework::render::{
        RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    };
    #[test]
    fn absent_instances_keep_duplicate_raw_rows_and_explicit_empty_stays_empty() {
        let extract = RenderVirtualGeometryExtract {
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity: 2,
                    cluster_id: 20,
                    page_id: 300,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 2,
                    cluster_id: 20,
                    page_id: 300,
                    ..Default::default()
                },
                RenderVirtualGeometryCluster {
                    entity: 3,
                    cluster_id: 30,
                    page_id: 301,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let (rows, provenance) = build_lossless_candidate_catalog(
            &extract,
            &BTreeSet::from([300]),
            &BTreeSet::new(),
            &BTreeSet::from([2, 3]),
        )
        .expect("raw authored catalog");
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| row.instance_index.is_none()));
        assert_eq!(
            provenance,
            vec![(u32::MAX, 0, 1), (u32::MAX, 1, 1), (u32::MAX, 2, 1)]
        );
        let (empty, empty_provenance) =
            paired_selection_catalog(&extract, &[]).expect("explicit zero candidates");
        assert!(empty.is_empty());
        assert!(empty_provenance.is_empty());
        assert_eq!(extract.clusters.len(), 3);
    }
}
