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
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
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
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 300,
                cluster_start_ordinal: 2,
                cluster_span_count: 1,
                cluster_total_count: 3,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
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
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 10,
                page_id: 200,
                cluster_start_ordinal: 1,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lod_level: 0,
                resident_slot: Some(0),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        "expected unified indirect ownership to respect the explicit prepare draw-segment boundaries instead of compacting them again in the renderer path"
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
        }]
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
