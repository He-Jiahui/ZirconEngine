use zircon_scene::{RenderVirtualGeometryExtract, RenderVirtualGeometryPage};

use crate::{
    runtime::{
        VirtualGeometryPageRequest, VirtualGeometryPageResidencyState, VirtualGeometryRuntimeState,
    },
    types::{
        VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPrepareDrawSegment, VirtualGeometryPrepareFrame,
        VirtualGeometryPrepareIndirectDraw, VirtualGeometryPreparePage,
        VirtualGeometryPrepareRequest,
    },
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

#[test]
fn virtual_geometry_runtime_state_tracks_page_table_and_request_sink() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        },
    );

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(500), Some(1));
    assert_eq!(state.page_slot(300), None);
    assert_eq!(
        state.page_residency(200),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.page_residency(500),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::PendingUpload)
    );
    assert_eq!(
        state.pending_requests(),
        vec![VirtualGeometryPageRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 7,
        }]
    );
    assert_eq!(state.evictable_pages(), vec![500]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.page_table_entry_count, 2);
    assert_eq!(snapshot.resident_page_count, 2);
    assert_eq!(snapshot.pending_request_count, 1);
}

#[test]
fn virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        },
    );
    state.ingest_plan(
        8,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: Vec::new(),
            evictable_pages: vec![500],
        },
    );

    assert_eq!(state.pending_requests().len(), 1);
    state.apply_evictions([500]);
    state.fulfill_requests([300]);

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), Some(1));
    assert_eq!(state.page_slot(500), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new()
    );

    let snapshot = state.snapshot();
    assert_eq!(snapshot.page_table_entry_count, 2);
    assert_eq!(snapshot.resident_page_count, 2);
    assert_eq!(snapshot.pending_request_count, 0);
}

#[test]
fn virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters()
{
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(400, false, 8192),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200],
            requested_pages: vec![300, 400],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![200],
        },
    );

    let prepare = state.build_prepare_frame(&[
        cluster(10, 1, 200, 0, 3),
        cluster(20, 2, 300, 1, 3),
        cluster(30, 3, 400, 2, 3),
    ]);

    assert_eq!(prepare.visible_entities, vec![10, 20]);
    assert_eq!(
        prepare.visible_clusters,
        vec![
            VirtualGeometryPrepareCluster {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: 20,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryPrepareCluster {
                entity: 30,
                cluster_id: 3,
                page_id: 400,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::Missing,
            },
        ]
    );
    assert_eq!(
        prepare.resident_pages,
        vec![VirtualGeometryPreparePage {
            page_id: 200,
            slot: 0,
            size_bytes: 2048,
        }]
    );
    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 7,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: None,
        }]
    );
    assert_eq!(prepare.available_slots, vec![1]);
    assert_eq!(
        prepare.evictable_pages,
        vec![VirtualGeometryPreparePage {
            page_id: 200,
            slot: 0,
            size_bytes: 2048,
        }]
    );
    assert_eq!(
        prepare.cluster_draw_segments,
        vec![
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                resident_slot: Some(0),
                cluster_ordinal: 0,
                cluster_span_count: 1,
                cluster_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareDrawSegment {
                entity: 20,
                cluster_id: 2,
                page_id: 300,
                resident_slot: None,
                cluster_ordinal: 1,
                cluster_span_count: 1,
                cluster_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ]
    );
}

#[test]
fn virtual_geometry_runtime_state_builds_visibility_owned_compacted_draw_segments() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![page(200, true, 2048), page(300, false, 4096)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![200],
        },
    );

    let prepare = state.build_prepare_frame(&[
        cluster(10, 1, 200, 0, 3),
        cluster(10, 2, 200, 1, 3),
        cluster(10, 3, 300, 2, 3),
    ]);

    assert_eq!(
        prepare.cluster_draw_segments,
        vec![
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                resident_slot: Some(0),
                cluster_ordinal: 0,
                cluster_span_count: 2,
                cluster_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 3,
                page_id: 300,
                resident_slot: None,
                cluster_ordinal: 2,
                cluster_span_count: 1,
                cluster_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ]
    );
    assert_eq!(
        prepare.unified_indirect_draws(),
        vec![
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 200,
                cluster_start_ordinal: 0,
                cluster_span_count: 2,
                cluster_total_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 300,
                cluster_start_ordinal: 2,
                cluster_span_count: 1,
                cluster_total_count: 3,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: Some(1),
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ]
    );
}

#[test]
fn virtual_geometry_runtime_state_preserves_visibility_owned_draw_segments_across_parent_lineages()
{
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 1,
        clusters: Vec::new(),
        pages: vec![page(400, true, 4096)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![400],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame_with_segments(
        &[cluster(10, 40, 400, 3, 5), cluster(10, 50, 400, 4, 5)],
        &[
            visibility_draw_segment(10, 40, 400, 3, 1, 5, 2),
            visibility_draw_segment(10, 50, 400, 4, 1, 5, 2),
        ],
    );

    assert_eq!(
        prepare.cluster_draw_segments,
        vec![
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 40,
                page_id: 400,
                resident_slot: Some(0),
                cluster_ordinal: 3,
                cluster_span_count: 1,
                cluster_count: 5,
                lineage_depth: 2,
                lod_level: 2,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 50,
                page_id: 400,
                resident_slot: Some(0),
                cluster_ordinal: 4,
                cluster_span_count: 1,
                cluster_count: 5,
                lineage_depth: 2,
                lod_level: 2,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        "expected runtime prepare to preserve visibility-owned lineage segment boundaries instead of compacting same-page resident clusters back into one segment"
    );
    assert_eq!(
        prepare.unified_indirect_draws(),
        vec![
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 400,
                cluster_start_ordinal: 3,
                cluster_span_count: 1,
                cluster_total_count: 5,
                lineage_depth: 2,
                lod_level: 2,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 400,
                cluster_start_ordinal: 4,
                cluster_span_count: 1,
                cluster_total_count: 5,
                lineage_depth: 2,
                lod_level: 2,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ]
    );
}

#[test]
fn virtual_geometry_prepare_frame_preserves_explicit_draw_segment_boundaries_in_unified_indirect_draws(
) {
    let prepare = VirtualGeometryPrepareFrame {
        visible_entities: vec![10],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: 10,
                cluster_id: 2,
                page_id: 200,
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        cluster_draw_segments: vec![
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                resident_slot: Some(0),
                cluster_ordinal: 0,
                cluster_span_count: 1,
                cluster_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 2,
                page_id: 200,
                resident_slot: Some(0),
                cluster_ordinal: 1,
                cluster_span_count: 1,
                cluster_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 200,
            slot: 0,
            size_bytes: 2048,
        }],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        prepare.unified_indirect_draws(),
        vec![
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 200,
                cluster_start_ordinal: 0,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 200,
                cluster_start_ordinal: 1,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        "expected unified indirect ownership to respect the explicit prepare draw-segment boundaries instead of compacting them again in the renderer path"
    );
}

#[test]
fn virtual_geometry_prepare_frame_sorts_unified_indirect_draws_by_submission_authority() {
    let prepare = VirtualGeometryPrepareFrame {
        visible_entities: vec![10],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: 10,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        cluster_draw_segments: vec![
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 2,
                page_id: 300,
                resident_slot: None,
                cluster_ordinal: 1,
                cluster_span_count: 1,
                cluster_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryPrepareDrawSegment {
                entity: 10,
                cluster_id: 1,
                page_id: 200,
                resident_slot: Some(0),
                cluster_ordinal: 0,
                cluster_span_count: 1,
                cluster_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 200,
            slot: 0,
            size_bytes: 2048,
        }],
        pending_page_requests: vec![VirtualGeometryPrepareRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 1,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: None,
        }],
        available_slots: vec![1],
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        prepare.unified_indirect_draws(),
        vec![
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 200,
                cluster_start_ordinal: 0,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: Some(0),
                submission_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 300,
                cluster_start_ordinal: 1,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: Some(1),
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        "expected prepare-owned unified indirect draws to be sorted by submission-slot/frontier authority even when the explicit draw-segment input arrives in a different order, so renderer-side mesh build no longer has to invent the first authoritative ordering"
    );
}

#[test]
fn virtual_geometry_runtime_state_consumes_feedback_and_promotes_requested_pages() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![2],
        requested_pages: vec![300],
        evictable_pages: vec![500],
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), Some(1));
    assert_eq!(state.page_slot(500), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new()
    );

    let prepare =
        state.build_prepare_frame(&[cluster(20, 2, 300, 1, 3), cluster(50, 5, 500, 2, 3)]);
    assert_eq!(prepare.visible_entities, vec![20]);
    assert_eq!(
        prepare.visible_clusters,
        vec![
            VirtualGeometryPrepareCluster {
                entity: 20,
                cluster_id: 2,
                page_id: 300,
                lod_level: 0,
                resident_slot: Some(1),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: 50,
                cluster_id: 5,
                page_id: 500,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::Missing,
            },
        ]
    );
}

#[test]
fn virtual_geometry_runtime_state_leaves_requests_pending_without_evictable_budget() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: Vec::new(),
        pages: vec![page(200, true, 2048), page(300, false, 4096)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: Vec::new(),
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![2],
        requested_pages: vec![300],
        evictable_pages: Vec::new(),
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::PendingUpload)
    );
    assert_eq!(
        state.pending_requests(),
        vec![VirtualGeometryPageRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 7,
        }]
    );
}

#[test]
fn virtual_geometry_runtime_state_applies_gpu_completed_pages_with_evictable_slots() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        },
    );

    state.complete_gpu_uploads_with_slots([(300, 1)], &[500]);

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), Some(1));
    assert_eq!(state.page_slot(500), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new()
    );
}

#[test]
fn virtual_geometry_runtime_state_rejects_gpu_slot_recycling_when_current_evictable_set_withdraws_page(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300],
            dirty_requested_pages: vec![300],
            evictable_pages: vec![500],
        },
    );

    state.complete_gpu_uploads_with_slots([(300, 1)], &[]);

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(500), Some(1));
    assert_eq!(state.page_slot(300), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::PendingUpload)
    );
    assert_eq!(
        state.pending_requests(),
        vec![VirtualGeometryPageRequest {
            page_id: 300,
            size_bytes: 4096,
            generation: 7,
        }],
        "expected runtime residency completion to obey the current visibility-owned evictable set instead of recycling a slot from stale runtime state"
    );
}

#[test]
fn virtual_geometry_runtime_state_applies_gpu_assigned_free_slots_before_evictable_recycling() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
            page(600, false, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300, 600],
            dirty_requested_pages: vec![300, 600],
            evictable_pages: vec![500],
        },
    );
    state.apply_evictions([500]);
    state.fulfill_requests([500]);
    state.apply_evictions([500]);

    let prepare = state.build_prepare_frame(&[]);
    assert_eq!(prepare.available_slots, vec![1, 2]);

    state.complete_gpu_uploads_with_slots([(300, 2), (600, 1)], &[500]);

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), Some(2));
    assert_eq!(state.page_slot(600), Some(1));
    assert_eq!(state.page_slot(500), None);
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new()
    );
}

#[test]
fn virtual_geometry_runtime_state_consumes_explicit_gpu_replacement_truth_before_slot_fallbacks() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, Some(20)),
            render_cluster(80, 800, Some(40)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 2048),
            page(400, true, 2048),
            page(800, true, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        91,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400],
        },
    );

    state.complete_gpu_uploads_with_replacements([(200, 2)], [(200, 800)], &[400]);

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(2));
    assert_eq!(state.page_slot(400), Some(1));
    assert_eq!(
        state.page_slot(800),
        None,
        "expected runtime completion to trust explicit GPU replacement truth and evict the reported recycled page even when the current evictable set no longer lists that page"
    );
    assert_eq!(
        state.page_residency(200),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new()
    );
}

#[test]
fn virtual_geometry_runtime_state_applies_gpu_page_table_snapshot_as_residency_truth() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: Vec::new(),
        pages: vec![
            page(200, true, 2048),
            page(300, false, 4096),
            page(500, true, 1024),
            page(600, false, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![300, 600],
            dirty_requested_pages: vec![300, 600],
            evictable_pages: vec![500],
        },
    );

    state.apply_gpu_page_table_entries(&[(200, 0), (300, 1)]);

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), Some(1));
    assert_eq!(state.page_slot(500), None);
    assert_eq!(state.page_slot(600), None);
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::Resident)
    );
    assert_eq!(
        state.page_residency(600),
        Some(VirtualGeometryPageResidencyState::PendingUpload)
    );
    assert_eq!(
        state.pending_requests(),
        vec![VirtualGeometryPageRequest {
            page_id: 600,
            size_bytes: 1024,
            generation: 7,
        }]
    );
    assert!(state.evictable_pages().is_empty());

    let prepare = state.build_prepare_frame(&[
        cluster(20, 2, 300, 1, 4),
        cluster(50, 5, 500, 2, 4),
        cluster(60, 6, 600, 3, 4),
    ]);
    assert_eq!(
        prepare.resident_pages,
        vec![
            VirtualGeometryPreparePage {
                page_id: 200,
                slot: 0,
                size_bytes: 2048,
            },
            VirtualGeometryPreparePage {
                page_id: 300,
                slot: 1,
                size_bytes: 4096,
            },
        ]
    );
    assert_eq!(prepare.available_slots, vec![2]);
    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 600,
            size_bytes: 1024,
            generation: 7,
            frontier_rank: 1,
            assigned_slot: Some(2),
            recycled_page_id: None,
        }]
    );
}

#[test]
fn virtual_geometry_runtime_state_drops_stale_scene_pages_and_pending_requests_when_extract_shrinks(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let initial_extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: Vec::new(),
        pages: vec![
            page(100, true, 2048),
            page(200, false, 4096),
            page(300, true, 1024),
        ],
    };

    state.register_extract(Some(&initial_extract));
    state.ingest_plan(
        11,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 300],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![300],
        },
    );
    state.complete_gpu_uploads_with_slots([(200, 2)], &[300]);

    let shrunk_extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: Vec::new(),
        pages: vec![page(100, true, 2048)],
    };

    state.register_extract(Some(&shrunk_extract));

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), None);
    assert_eq!(state.page_slot(300), None);
    assert_eq!(
        state.page_residency(200),
        None,
        "expected runtime residency state to purge removed scene pages instead of keeping stale resident uploads alive"
    );
    assert_eq!(
        state.page_residency(300),
        None,
        "expected runtime residency state to evict removed resident pages when the extract shrinks"
    );
    assert_eq!(
        state.pending_requests(),
        Vec::<VirtualGeometryPageRequest>::new(),
        "expected removed pages to disappear from the pending uploader queue"
    );
    assert_eq!(state.evictable_pages(), Vec::<u32>::new());

    let prepare = state.build_prepare_frame(&[
        cluster(10, 1, 100, 0, 1),
        cluster(20, 2, 200, 1, 2),
        cluster(30, 3, 300, 2, 3),
    ]);
    assert_eq!(
        prepare.resident_pages,
        vec![VirtualGeometryPreparePage {
            page_id: 100,
            slot: 0,
            size_bytes: 2048,
        }]
    );
    assert_eq!(prepare.pending_page_requests, Vec::new());
    assert_eq!(prepare.available_slots, Vec::<u32>::new());
    assert_eq!(prepare.evictable_pages, Vec::new());
}

#[test]
fn virtual_geometry_runtime_state_withholds_descendant_page_requests_while_ancestor_upload_remains_pending(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 2,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(30, 300, Some(20)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 8192),
            page(300, false, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        21,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100],
            requested_pages: vec![200, 300],
            dirty_requested_pages: vec![200, 300],
            evictable_pages: Vec::new(),
        },
    );

    let blocked_prepare = state.build_prepare_frame(&[]);
    assert_eq!(
        blocked_prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 200,
            size_bytes: 8192,
            generation: 21,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: None,
        }],
        "expected runtime prepare to withhold descendant page uploads while the missing ancestor page is still pending so residency cascades do not bypass the collapsed hierarchy"
    );

    state.complete_gpu_uploads_with_slots([(200, 1)], &[]);

    let unblocked_prepare = state.build_prepare_frame(&[]);
    assert_eq!(
        unblocked_prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 300,
            size_bytes: 2048,
            generation: 21,
            frontier_rank: 1,
            assigned_slot: None,
            recycled_page_id: None,
        }],
        "expected descendant page uploads to re-enter the prepare queue once their pending ancestor page becomes resident"
    );
}

#[test]
fn virtual_geometry_runtime_state_prioritizes_pending_ancestor_pages_that_reconnect_hot_descendants(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, Some(20)),
            render_cluster(80, 800, Some(10)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 8192),
            page(400, true, 2048),
            page(800, false, 4096),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        31,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400],
            requested_pages: vec![200, 800],
            dirty_requested_pages: vec![800, 200],
            evictable_pages: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame(&[]);
    assert_eq!(
        prepare.pending_page_requests,
        vec![
            VirtualGeometryPrepareRequest {
                page_id: 200,
                size_bytes: 8192,
                generation: 31,
                frontier_rank: 0,
                assigned_slot: None,
                recycled_page_id: None,
            },
            VirtualGeometryPrepareRequest {
                page_id: 800,
                size_bytes: 4096,
                generation: 31,
                frontier_rank: 1,
                assigned_slot: None,
                recycled_page_id: None,
            },
        ],
        "expected runtime prepare to prioritize the missing ancestor page that reconnects already-resident descendants before unrelated pending uploads so residency cascades converge instead of thrashing hot descendant lineages"
    );
}

#[test]
fn virtual_geometry_runtime_state_prefers_evicting_unrelated_pages_before_target_ancestors() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 2,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(30, 300, None),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 4096),
            page(300, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        41,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 300],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![100, 300],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![20],
        requested_pages: vec![200],
        evictable_pages: vec![100, 300],
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(1));
    assert_eq!(state.page_slot(300), None);
    assert_eq!(
        state.page_residency(100),
        Some(VirtualGeometryPageResidencyState::Resident),
        "expected runtime residency to keep the target ancestor page hot and evict an unrelated page first when fulfilling a child-page request under tight budget"
    );
}

#[test]
fn virtual_geometry_runtime_state_prefers_evicting_unrelated_pages_before_target_descendants_for_gpu_assignment(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, Some(20)),
            render_cluster(80, 800, None),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 8192),
            page(400, true, 2048),
            page(800, true, 1024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        51,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    state.complete_gpu_uploads_with_slots([(200, 3)], &[400, 800]);

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(400), Some(1));
    assert_eq!(state.page_slot(800), None);
    assert_eq!(state.page_slot(200), Some(3));
    assert_eq!(
        state.page_residency(400),
        Some(VirtualGeometryPageResidencyState::Resident),
        "expected GPU slot assignment to preserve hot descendant residency and evict an unrelated page first when reconnecting an ancestor page into the same lineage"
    );
}

#[test]
fn virtual_geometry_runtime_state_prefers_evicting_farther_target_ancestors_before_nearer_ones() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(30, 300, Some(20)),
            render_cluster(40, 400, Some(30)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, true, 2048),
            page(300, true, 2048),
            page(400, false, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        61,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 300],
            requested_pages: vec![400],
            dirty_requested_pages: vec![400],
            evictable_pages: vec![100, 200, 300],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![40],
        requested_pages: vec![400],
        evictable_pages: vec![100, 200, 300],
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(100), None);
    assert_eq!(state.page_slot(200), Some(1));
    assert_eq!(state.page_slot(300), Some(2));
    assert_eq!(state.page_slot(400), Some(0));
    assert_eq!(
        state.page_residency(200),
        Some(VirtualGeometryPageResidencyState::Resident),
        "expected runtime residency to evict the farthest ancestor page before a nearer ancestor so deeper split-merge cascades keep the immediate lineage hot"
    );
    assert_eq!(
        state.page_residency(300),
        Some(VirtualGeometryPageResidencyState::Resident),
        "expected runtime residency to preserve the nearest ancestor page while reconnecting the missing child page into the same lineage"
    );
}

#[test]
fn virtual_geometry_runtime_state_prefers_evicting_farther_target_descendants_before_nearer_ones_for_gpu_assignment(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, Some(20)),
            render_cluster(80, 800, Some(40)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 2048),
            page(400, true, 2048),
            page(800, true, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        71,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    state.complete_gpu_uploads_with_slots([(200, 3)], &[400, 800]);

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(3));
    assert_eq!(state.page_slot(400), Some(1));
    assert_eq!(state.page_slot(800), None);
    assert_eq!(
        state.page_residency(400),
        Some(VirtualGeometryPageResidencyState::Resident),
        "expected GPU slot assignment to keep the nearer descendant hot and recycle the farther descendant first so deeper residency cascades do not thrash the active frontier"
    );
}

#[test]
fn virtual_geometry_runtime_state_builds_prepare_requests_with_explicit_frontier_assigned_recycle_slots(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, Some(20)),
            render_cluster(80, 800, Some(40)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 2048),
            page(400, true, 2048),
            page(800, true, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        81,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    let prepare = state.build_prepare_frame(&[]);
    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 200,
            size_bytes: 2048,
            generation: 81,
            frontier_rank: 0,
            assigned_slot: Some(2),
            recycled_page_id: Some(800),
        }],
        "expected prepare-time request contracts to carry the frontier-aware recycled slot explicitly so GPU uploader/page-table completion do not fall back to raw evictable-page input order"
    );
}

#[test]
fn virtual_geometry_runtime_state_keeps_frontier_recycle_preference_for_later_requests_without_assigned_slots(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 3,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(30, 300, Some(10)),
            render_cluster(40, 400, Some(30)),
            render_cluster(50, 500, Some(10)),
            render_cluster(80, 800, Some(10)),
        ],
        pages: vec![
            page(100, true, 2048),
            page(200, false, 2048),
            page(300, false, 2048),
            page(400, true, 2048),
            page(500, false, 2048),
            page(800, true, 2048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        82,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200, 300, 500],
            dirty_requested_pages: vec![200, 300, 500],
            evictable_pages: vec![400, 800],
        },
    );

    let prepare = state.build_prepare_frame(&[]);
    assert_eq!(
        prepare.pending_page_requests,
        vec![
            VirtualGeometryPrepareRequest {
                page_id: 200,
                size_bytes: 2048,
                generation: 82,
                frontier_rank: 0,
                assigned_slot: Some(2),
                recycled_page_id: Some(800),
            },
            VirtualGeometryPrepareRequest {
                page_id: 300,
                size_bytes: 2048,
                generation: 82,
                frontier_rank: 1,
                assigned_slot: Some(1),
                recycled_page_id: Some(400),
            },
            VirtualGeometryPrepareRequest {
                page_id: 500,
                size_bytes: 2048,
                generation: 82,
                frontier_rank: 2,
                assigned_slot: None,
                recycled_page_id: Some(800),
            },
        ],
        "expected later pending uploads without an immediately assignable slot to keep their frontier-aware recycle preference so GPU fallback submission can still reuse the colder lineage slot if earlier requests are skipped"
    );
}

fn page(page_id: u32, resident: bool, size_bytes: u64) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes,
    }
}

fn cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    cluster_ordinal: u32,
    cluster_count: u32,
) -> VisibilityVirtualGeometryCluster {
    VisibilityVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level: 0,
        cluster_ordinal,
        cluster_count,
        resident: page_id == 200,
    }
}

fn visibility_draw_segment(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    cluster_ordinal: u32,
    cluster_span_count: u32,
    cluster_count: u32,
    lod_level: u8,
) -> VisibilityVirtualGeometryDrawSegment {
    VisibilityVirtualGeometryDrawSegment {
        entity,
        cluster_id,
        page_id,
        cluster_ordinal,
        cluster_span_count,
        cluster_count,
        lineage_depth: u32::from(lod_level),
        lod_level,
    }
}

fn render_cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
) -> zircon_scene::RenderVirtualGeometryCluster {
    zircon_scene::RenderVirtualGeometryCluster {
        entity: 10,
        cluster_id,
        page_id,
        lod_level: 0,
        parent_cluster_id,
        bounds_center: zircon_math::Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}
