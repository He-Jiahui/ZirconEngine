use std::collections::BTreeSet;

use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
};
use crate::graphics::types::VirtualGeometryPrepareFrame;

use super::super::frame_submission_context::FrameSubmissionContext;

pub(super) fn build_virtual_geometry_debug_snapshot(
    context: &FrameSubmissionContext,
    prepare: Option<&VirtualGeometryPrepareFrame>,
) -> Option<RenderVirtualGeometryDebugSnapshot> {
    let extract = context.virtual_geometry_extract.as_ref()?;
    let page_upload_plan = context
        .virtual_geometry_page_upload_plan
        .clone()
        .unwrap_or_default();
    let feedback = context
        .virtual_geometry_feedback
        .clone()
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
        .then(|| context.virtual_geometry_bvh_visualization_instances.clone())
        .unwrap_or_default();
    let selected_clusters = prepare
        .and_then(|prepare| {
            let selected_clusters = prepare.selected_clusters(extract);
            (!selected_clusters.is_empty()).then_some(selected_clusters)
        })
        .unwrap_or_else(|| {
            build_selected_clusters_from_visibility_feedback(
                extract,
                &visible_cluster_id_set,
                &resident_page_set,
                &requested_page_set,
            )
        });
    let visbuffer_debug_marks = extract
        .debug
        .visualize_visbuffer
        .then(|| build_visbuffer_debug_marks_from_selected_clusters(&selected_clusters))
        .unwrap_or_default();
    let visbuffer64_entries = build_visbuffer64_entries_from_selected_clusters(&selected_clusters);
    let resident_page_inspections = prepare
        .map(|prepare| {
            prepare
                .resident_pages
                .iter()
                .map(|page| RenderVirtualGeometryResidentPageInspection {
                    page_id: page.page_id,
                    slot: page.slot,
                    size_bytes: page.size_bytes,
                })
                .collect()
        })
        .unwrap_or_default();
    let pending_page_request_inspections = prepare
        .map(|prepare| {
            prepare
                .pending_page_requests
                .iter()
                .map(|request| RenderVirtualGeometryPageRequestInspection {
                    page_id: request.page_id,
                    size_bytes: request.size_bytes,
                    generation: request.generation,
                    frontier_rank: request.frontier_rank,
                    assigned_slot: request.assigned_slot,
                    recycled_page_id: request.recycled_page_id,
                })
                .collect()
        })
        .unwrap_or_default();
    let available_page_slots = prepare
        .map(|prepare| prepare.available_slots.clone())
        .unwrap_or_default();
    let evictable_page_inspections = prepare
        .map(|prepare| {
            prepare
                .evictable_pages
                .iter()
                .map(|page| RenderVirtualGeometryResidentPageInspection {
                    page_id: page.page_id,
                    slot: page.slot,
                    size_bytes: page.size_bytes,
                })
                .collect()
        })
        .unwrap_or_default();

    Some(RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        debug: extract.debug,
        cull_input: build_cull_input_snapshot(extract, prepare),
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        cpu_reference_instances: context.virtual_geometry_cpu_reference_instances.clone(),
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
    prepare: Option<&VirtualGeometryPrepareFrame>,
) -> RenderVirtualGeometryCullInputSnapshot {
    RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: extract.cluster_budget,
        page_budget: extract.page_budget,
        instance_count: saturated_u32_len(extract.instances.len()),
        cluster_count: saturated_u32_len(extract.clusters.len()),
        page_count: saturated_u32_len(extract.pages.len()),
        visible_entity_count: prepare
            .map(|prepare| saturated_u32_len(prepare.visible_entities.len()))
            .unwrap_or_else(|| unique_extract_entity_count(extract)),
        visible_cluster_count: prepare
            .map(|prepare| saturated_u32_len(prepare.visible_clusters.len()))
            .unwrap_or_else(|| saturated_u32_len(extract.clusters.len())),
        resident_page_count: prepare
            .map(|prepare| saturated_u32_len(prepare.resident_pages.len()))
            .unwrap_or(0),
        pending_page_request_count: prepare
            .map(|prepare| saturated_u32_len(prepare.pending_page_requests.len()))
            .unwrap_or(0),
        available_page_slot_count: prepare
            .map(|prepare| saturated_u32_len(prepare.available_slots.len()))
            .unwrap_or(0),
        evictable_page_count: prepare
            .map(|prepare| saturated_u32_len(prepare.evictable_pages.len()))
            .unwrap_or(0),
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

#[cfg(test)]
mod tests {
    use super::build_virtual_geometry_debug_snapshot;
    use crate::core::framework::render::{
        RenderPipelineHandle, RenderVirtualGeometryCluster,
        RenderVirtualGeometryClusterSelectionInputSource,
        RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryDebugState,
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance, RenderVirtualGeometryVisBufferMark,
    };
    use crate::core::math::{Transform, UVec2, Vec3};
    use crate::graphics::runtime::render_framework::submit_frame_extract::frame_submission_context::{
        FrameSubmissionContext, HybridGiSceneInputs, UiSubmissionStats,
    };
    use crate::graphics::types::{VirtualGeometryPrepareFrame, VirtualGeometryPreparePage};
    use crate::scene::world::World;
    use crate::{
        RenderPipelineAsset, RenderPipelineCompileOptions, VisibilityContext,
        VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
    };

    #[test]
    fn debug_snapshot_prefers_prepare_owned_same_frame_visbuffer_marks_when_available() {
        let entity = 77_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 2,
            page_budget: 1,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 20,
                    hierarchy_node_id: None,
                    page_id: 200,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 30,
                    hierarchy_node_id: None,
                    page_id: 300,
                    lod_level: 10,
                    parent_cluster_id: Some(20),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("SnapshotVisBufferUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState {
                visualize_visbuffer: true,
                ..RenderVirtualGeometryDebugState::default()
            },
        };
        let context = frame_submission_context(
            extract.clone(),
            VisibilityVirtualGeometryFeedback {
                visible_cluster_ids: vec![20, 30],
                requested_pages: vec![300],
                evictable_pages: Vec::new(),
                hot_resident_pages: Vec::new(),
            },
            VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200],
                requested_pages: vec![300],
                dirty_requested_pages: vec![300],
                evictable_pages: Vec::new(),
            },
        );
        let prepare = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: Vec::new(),
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 200,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: Vec::new(),
            available_slots: Vec::new(),
            evictable_pages: Vec::new(),
        };

        let snapshot = build_virtual_geometry_debug_snapshot(&context, Some(&prepare))
            .expect("expected virtual geometry snapshot");

        assert_eq!(
            snapshot.visbuffer_debug_marks,
            vec![RenderVirtualGeometryVisBufferMark {
                instance_index: Some(0),
                entity,
                cluster_id: 20,
                page_id: 200,
                lod_level: 10,
                state: RenderVirtualGeometryExecutionState::Resident,
                color_rgba: [92, 190, 130, 255],
            }],
            "expected submission-build visbuffer snapshot marks to prefer prepare-owned same-frame truth instead of broad visibility feedback when prepare already projected the authoritative current-frame draw subset"
        );
    }

    #[test]
    fn debug_snapshot_builds_cull_input_snapshot_from_extract_and_prepare_state() {
        let entity = 77_u64;
        let extract = RenderVirtualGeometryExtract {
            cluster_budget: 4,
            page_budget: 3,
            clusters: vec![
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 20,
                    hierarchy_node_id: None,
                    page_id: 200,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    screen_space_error: 0.25,
                },
                RenderVirtualGeometryCluster {
                    entity,
                    cluster_id: 30,
                    hierarchy_node_id: None,
                    page_id: 300,
                    lod_level: 9,
                    parent_cluster_id: Some(20),
                    bounds_center: Vec3::X,
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                },
            ],
            hierarchy_nodes: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 0,
                mesh_name: Some("SnapshotCullInputUnitTest".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(9),
                freeze_cull: true,
                visualize_bvh: true,
                visualize_visbuffer: false,
                print_leaf_clusters: true,
            },
        };
        let context = frame_submission_context(
            extract.clone(),
            VisibilityVirtualGeometryFeedback {
                visible_cluster_ids: vec![20, 30],
                requested_pages: vec![300],
                evictable_pages: vec![200],
                hot_resident_pages: vec![200],
            },
            VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![200],
                requested_pages: vec![300],
                dirty_requested_pages: vec![300],
                evictable_pages: vec![200],
            },
        );
        let prepare = VirtualGeometryPrepareFrame {
            visible_entities: vec![entity],
            visible_clusters: vec![
                crate::graphics::types::VirtualGeometryPrepareCluster {
                    entity,
                    cluster_id: 20,
                    page_id: 200,
                    lod_level: 10,
                    resident_slot: Some(0),
                    state: crate::graphics::types::VirtualGeometryPrepareClusterState::Resident,
                },
                crate::graphics::types::VirtualGeometryPrepareCluster {
                    entity,
                    cluster_id: 30,
                    page_id: 300,
                    lod_level: 9,
                    resident_slot: None,
                    state:
                        crate::graphics::types::VirtualGeometryPrepareClusterState::PendingUpload,
                },
            ],
            cluster_draw_segments: Vec::new(),
            resident_pages: vec![VirtualGeometryPreparePage {
                page_id: 200,
                slot: 0,
                size_bytes: 4096,
            }],
            pending_page_requests: vec![crate::graphics::types::VirtualGeometryPrepareRequest {
                page_id: 300,
                size_bytes: 4096,
                generation: 1,
                frontier_rank: 0,
                assigned_slot: Some(1),
                recycled_page_id: None,
            }],
            available_slots: vec![2],
            evictable_pages: vec![VirtualGeometryPreparePage {
                page_id: 200,
                slot: 0,
                size_bytes: 4096,
            }],
        };

        let snapshot = build_virtual_geometry_debug_snapshot(&context, Some(&prepare))
            .expect("expected virtual geometry snapshot");

        assert_eq!(
            snapshot.cull_input,
            RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 4,
                page_budget: 3,
                instance_count: 1,
                cluster_count: 2,
                page_count: 0,
                visible_entity_count: 1,
                visible_cluster_count: 2,
                resident_page_count: 1,
                pending_page_request_count: 1,
                available_page_slot_count: 1,
                evictable_page_count: 1,
                debug: extract.debug,
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
            },
            "expected the submission-build snapshot to expose one stable cull-input DTO with extract budgets, current prepare-frontier counts, and the pre-store cluster-selection provenance placeholder that later render-path storage patches to the frame-owned source"
        );
    }

    fn frame_submission_context(
        extract: RenderVirtualGeometryExtract,
        feedback: VisibilityVirtualGeometryFeedback,
        page_upload_plan: VisibilityVirtualGeometryPageUploadPlan,
    ) -> FrameSubmissionContext {
        let mut frame_extract = World::new().to_render_frame_extract();
        frame_extract.apply_viewport_size(UVec2::new(32, 32));
        let compiled_pipeline = RenderPipelineAsset::default_forward_plus()
            .compile_with_options(&frame_extract, &RenderPipelineCompileOptions::default())
            .expect("expected test pipeline to compile");

        FrameSubmissionContext {
            size: UVec2::new(32, 32),
            pipeline_handle: RenderPipelineHandle::new(1),
            quality_profile: None,
            compiled_pipeline,
            visibility_context: VisibilityContext::default(),
            ui_stats: UiSubmissionStats::default(),
            previous_hybrid_gi_runtime: None,
            previous_virtual_geometry_runtime: None,
            hybrid_gi_enabled: false,
            virtual_geometry_enabled: true,
            hybrid_gi_extract: None,
            hybrid_gi_scene_inputs: HybridGiSceneInputs::default(),
            hybrid_gi_update_plan: None,
            hybrid_gi_feedback: None,
            virtual_geometry_extract: Some(extract),
            virtual_geometry_cpu_reference_instances: Vec::new(),
            virtual_geometry_bvh_visualization_instances: Vec::new(),
            virtual_geometry_page_upload_plan: Some(page_upload_plan),
            virtual_geometry_feedback: Some(feedback),
            predicted_generation: 1,
        }
    }
}
