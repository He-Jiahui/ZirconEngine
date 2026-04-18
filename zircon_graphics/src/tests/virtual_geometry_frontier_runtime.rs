use zircon_scene::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
};

use crate::{
    runtime::VirtualGeometryRuntimeState, types::VirtualGeometryPrepareRequest,
    VisibilityVirtualGeometryFeedback, VisibilityVirtualGeometryPageUploadPlan,
};

#[test]
fn virtual_geometry_runtime_state_uses_current_visibility_request_order_for_pending_uploads() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 4,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(80, 800, None),
            render_cluster(90, 900, Some(80)),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(800, false, 4_096),
            page(900, true, 1_024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        1,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![900],
            requested_pages: vec![800, 200],
            dirty_requested_pages: vec![800, 200],
            evictable_pages: Vec::new(),
        },
    );
    state.ingest_plan(
        2,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![900],
            requested_pages: vec![200, 800],
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![
            VirtualGeometryPrepareRequest {
                page_id: 200,
                size_bytes: 2_048,
                generation: 1,
                frontier_rank: 0,
                assigned_slot: Some(1),
                recycled_page_id: None,
            },
            VirtualGeometryPrepareRequest {
                page_id: 800,
                size_bytes: 4_096,
                generation: 1,
                frontier_rank: 1,
                assigned_slot: Some(2),
                recycled_page_id: None,
            },
        ],
        "expected runtime prepare to preserve the current visibility-owned request frontier order instead of letting legacy descendant-count heuristics re-rank an older pending upload ahead of the active frontier"
    );
}

#[test]
fn virtual_geometry_runtime_state_evicts_unrelated_pages_before_active_request_lineages() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 6,
        page_budget: 3,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, Some(20)),
            render_cluster(50, 500, None),
            render_cluster(60, 600, Some(50)),
            render_cluster(70, 700, None),
            render_cluster(90, 900, None),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 1_024),
            page(500, false, 2_048),
            page(600, true, 1_024),
            page(700, false, 2_048),
            page(900, true, 1_024),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        3,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 600, 900],
            requested_pages: vec![200, 500, 700],
            dirty_requested_pages: vec![200, 500, 700],
            evictable_pages: vec![600, 900, 300],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: vec![700],
        evictable_pages: vec![600, 900, 300],
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(300), Some(0));
    assert_eq!(state.page_slot(600), Some(1));
    assert_eq!(state.page_slot(900), None);
    assert_eq!(state.page_slot(700), Some(2));
    assert_eq!(
        state.evictable_pages(),
        vec![600, 300],
        "expected runtime residency to evict an unrelated page before pages that still anchor other active request lineages while a new frontier page is being promoted"
    );
}

#[test]
fn virtual_geometry_runtime_state_evicts_later_active_request_lineage_before_earlier_one() {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, Some(20)),
            render_cluster(50, 500, None),
            render_cluster(60, 600, Some(50)),
            render_cluster(70, 700, None),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 1_024),
            page(500, false, 2_048),
            page(600, true, 1_024),
            page(700, false, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        4,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 600],
            requested_pages: vec![200, 500, 700],
            dirty_requested_pages: vec![200, 500, 700],
            evictable_pages: vec![300, 600],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: vec![700],
        evictable_pages: vec![300, 600],
        hot_resident_pages: Vec::new(),
    });

    assert_eq!(state.page_slot(300), Some(0));
    assert_eq!(state.page_slot(600), None);
    assert_eq!(state.page_slot(700), Some(1));
    assert_eq!(
        state.evictable_pages(),
        vec![300],
        "expected runtime residency to preserve the earlier visibility-owned request lineage and recycle the later active lineage first when no unrelated page remains"
    );
}

#[test]
fn virtual_geometry_runtime_state_prefers_evicting_cold_page_before_recent_frontier_hot_page_during_feedback_completion(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(70, 700, None),
        ],
        pages: vec![
            page(200, true, 2_048),
            page(300, true, 2_048),
            page(700, false, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        5,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: vec![700],
            dirty_requested_pages: vec![700],
            evictable_pages: vec![200, 300],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: vec![700],
        evictable_pages: vec![200, 300],
        hot_resident_pages: vec![200],
    });

    assert_eq!(state.page_slot(200), Some(0));
    assert_eq!(state.page_slot(300), None);
    assert_eq!(state.page_slot(700), Some(1));
}

#[test]
fn virtual_geometry_runtime_state_carries_recent_frontier_hot_pages_into_next_prepare_recycle_plan()
{
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(70, 700, None),
        ],
        pages: vec![
            page(200, true, 2_048),
            page(300, true, 2_048),
            page(700, false, 4_096),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        6,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: vec![200],
    });
    state.ingest_plan(
        7,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: vec![700],
            dirty_requested_pages: vec![700],
            evictable_pages: vec![200, 300],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 700,
            size_bytes: 4_096,
            generation: 7,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: Some(300),
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

fn render_cluster(
    cluster_id: u32,
    page_id: u32,
    parent_cluster_id: Option<u32>,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
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
