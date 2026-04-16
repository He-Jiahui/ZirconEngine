use zircon_scene::{RenderVirtualGeometryExtract, RenderVirtualGeometryPage};

use crate::{
    runtime::{
        VirtualGeometryPageRequest, VirtualGeometryPageResidencyState, VirtualGeometryRuntimeState,
    },
    types::{
        VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
        VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
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
        cluster(10, 1, 200),
        cluster(20, 2, 300),
        cluster(30, 3, 400),
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
    assert_eq!(
        prepare.evictable_pages,
        vec![VirtualGeometryPreparePage {
            page_id: 200,
            slot: 0,
            size_bytes: 2048,
        }]
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

    let prepare = state.build_prepare_frame(&[cluster(20, 2, 300), cluster(50, 5, 500)]);
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

    state.complete_gpu_uploads([300], &[500]);

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

fn page(page_id: u32, resident: bool, size_bytes: u64) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes,
    }
}

fn cluster(entity: u64, cluster_id: u32, page_id: u32) -> VisibilityVirtualGeometryCluster {
    VisibilityVirtualGeometryCluster {
        entity,
        cluster_id,
        page_id,
        lod_level: 0,
        resident: page_id == 200,
    }
}
