use crate::core::framework::render::{
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
fn virtual_geometry_runtime_state_keeps_first_unique_visibility_request_order_when_duplicate_requested_pages_reappear(
) {
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
            requested_pages: vec![200, 800, 200],
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
        "expected duplicate requested page ids to preserve the first unique visibility-owned frontier order instead of letting a later duplicate overwrite the active request rank"
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
fn virtual_geometry_runtime_state_keeps_hot_later_request_lineage_resident_while_completing_new_target_page(
) {
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
        hot_resident_pages: vec![600],
    });

    assert_eq!(state.page_slot(300), None);
    assert_eq!(state.page_slot(600), Some(1));
    assert_eq!(state.page_slot(700), Some(0));
    assert_eq!(
        state.evictable_pages(),
        vec![600],
        "expected deeper frontier policy to keep a recently-hot later-request lineage resident and recycle the colder earlier lineage first while promoting the new target page"
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

#[test]
fn virtual_geometry_runtime_state_keeps_hot_farther_descendant_resident_while_reconnecting_missing_ancestor(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        8,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![200],
        evictable_pages: vec![400, 800],
        hot_resident_pages: vec![800],
    });

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(1));
    assert_eq!(state.page_slot(400), None);
    assert_eq!(state.page_slot(800), Some(2));
    assert_eq!(
        state.evictable_pages(),
        vec![800],
        "expected deeper split-merge residency to keep the recently-hot farther descendant resident and recycle the colder nearer descendant first while reconnecting the missing ancestor"
    );
}

#[test]
fn virtual_geometry_runtime_state_keeps_deepest_hot_descendant_resident_when_same_frontier_has_multiple_hot_descendants(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![40, 80],
        requested_pages: vec![200],
        evictable_pages: vec![400, 800],
        hot_resident_pages: vec![400, 800],
    });

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(1));
    assert_eq!(state.page_slot(400), None);
    assert_eq!(state.page_slot(800), Some(2));
    assert_eq!(
        state.evictable_pages(),
        vec![800],
        "expected deeper split-merge residency to preserve the deepest hot descendant frontier page and recycle the shallower hot descendant first while reconnecting a missing ancestor"
    );
}

#[test]
fn virtual_geometry_runtime_state_cascades_hot_frontier_ancestor_truth_to_deeper_descendant_recycle_policy(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );

    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![40],
        requested_pages: vec![200],
        evictable_pages: vec![400, 800],
        hot_resident_pages: vec![400],
    });

    assert_eq!(state.page_slot(100), Some(0));
    assert_eq!(state.page_slot(200), Some(1));
    assert_eq!(state.page_slot(400), None);
    assert_eq!(state.page_slot(800), Some(2));
    assert_eq!(
        state.evictable_pages(),
        vec![800],
        "expected confirmed hot-frontier truth on the ancestor page to keep cascading down the same split-merge lineage so the colder shallower descendant is recycled before the deeper descendant still anchored by that hot frontier branch"
    );
}

#[test]
fn virtual_geometry_runtime_state_carries_hot_frontier_truth_into_newly_completed_descendant_before_next_prepare(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, false, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        10,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400],
            requested_pages: vec![800],
            dirty_requested_pages: vec![800],
            evictable_pages: vec![400],
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![40],
        requested_pages: vec![800],
        evictable_pages: vec![400],
        hot_resident_pages: vec![400],
    });
    state.complete_gpu_uploads_with_replacements([(800, 2)], std::iter::empty(), &[400]);
    state.apply_gpu_page_table_entries(&[(100, 0), (400, 1), (800, 2)]);
    state.ingest_plan(
        11,
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
            size_bytes: 2_048,
            generation: 11,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: Some(400),
        }],
        "expected page-table-confirmed completion to carry hot-frontier truth into the newly completed descendant so the next prepare recycle plan keeps the deeper frontier resident instead of treating it as a cold page"
    );
}

#[test]
fn virtual_geometry_runtime_state_carries_hot_descendant_frontier_truth_into_newly_completed_ancestor_before_next_prepare(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 4,
        clusters: vec![
            render_cluster(10, 100, None),
            render_cluster(20, 200, Some(10)),
            render_cluster(40, 400, None),
            render_cluster(50, 500, None),
            render_cluster(80, 800, Some(20)),
        ],
        pages: vec![
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(500, false, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        12,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![200],
        evictable_pages: vec![400, 800],
        hot_resident_pages: vec![800],
    });
    state.complete_gpu_uploads_with_replacements([(200, 3)], std::iter::empty(), &[400, 800]);
    state.apply_gpu_page_table_entries(&[(100, 0), (400, 1), (800, 2), (200, 3)]);
    state.ingest_plan(
        13,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 200, 400, 800],
            requested_pages: vec![500],
            dirty_requested_pages: vec![500],
            evictable_pages: vec![200, 400, 800],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 500,
            size_bytes: 2_048,
            generation: 13,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: Some(400),
        }],
        "expected completion + page-table-confirmed ancestor promotion to inherit hot descendant frontier truth so the next prepare recycle plan keeps the newly reconnected lineage ancestor resident instead of immediately evicting it as a cold unrelated page"
    );
}

#[test]
fn virtual_geometry_runtime_state_carries_recent_hot_frontier_lineage_through_one_cooling_frame_before_next_prepare(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        16,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: vec![800],
    });
    state.ingest_plan(
        17,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: Vec::new(),
    });
    state.ingest_plan(
        18,
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
            size_bytes: 2_048,
            generation: 18,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: Some(400),
        }],
        "expected recent confirmed hot-frontier truth to survive one cooling feedback frame so the next prepare recycle plan still preserves the deeper split-merge descendant instead of immediately evicting it as soon as the current feedback stops marking it hot"
    );
}

#[test]
fn virtual_geometry_runtime_state_carries_confirmed_hot_frontier_lineage_through_two_cooling_frames_before_next_prepare(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        18,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: vec![800],
    });
    state.ingest_plan(
        19,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: Vec::new(),
    });
    state.ingest_plan(
        20,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: Vec::new(),
    });
    state.ingest_plan(
        21,
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
            size_bytes: 2_048,
            generation: 21,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: Some(400),
        }],
        "expected confirmed hot-frontier truth to survive two cooling feedback frames so the later reconnect prepare still preserves the deeper split-merge descendant instead of dropping back to a single-frame recent-hot bias"
    );
}

#[test]
fn virtual_geometry_runtime_state_drops_confirmed_hot_frontier_lineage_after_cooling_budget_expires(
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
            page(100, true, 2_048),
            page(200, false, 2_048),
            page(400, true, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        22,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![100, 400, 800],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: vec![800],
    });
    for generation in 23..=25 {
        state.ingest_plan(
            generation,
            &VisibilityVirtualGeometryPageUploadPlan {
                resident_pages: vec![100, 400, 800],
                requested_pages: Vec::new(),
                dirty_requested_pages: Vec::new(),
                evictable_pages: Vec::new(),
            },
        );
        state.consume_feedback(&VisibilityVirtualGeometryFeedback {
            visible_cluster_ids: Vec::new(),
            requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
            hot_resident_pages: Vec::new(),
        });
    }
    state.ingest_plan(
        26,
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
            size_bytes: 2_048,
            generation: 26,
            frontier_rank: 0,
            assigned_slot: Some(2),
            recycled_page_id: Some(800),
        }],
        "expected the deeper cooling cascade to stay bounded, so once the confirmed hot-frontier cooling budget expires the reconnect prepare drops back to the colder-depth ordering instead of keeping the old descendant hot indefinitely"
    );
}

#[test]
fn virtual_geometry_runtime_state_feedback_completion_carries_recent_frontier_truth_into_reconnected_ancestor_after_descendant_leaves_residency(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 6,
        page_budget: 3,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(40, 400, Some(20)),
            render_cluster(50, 500, None),
            render_cluster(60, 600, None),
            render_cluster(80, 800, Some(40)),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 2_048),
            page(400, true, 2_048),
            page(500, false, 2_048),
            page(600, false, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        19,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 400, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![400, 800],
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![200],
        evictable_pages: vec![400, 800],
        hot_resident_pages: vec![800],
    });
    state.apply_evictions([800]);
    state.ingest_plan(
        20,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300, 600],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: Vec::new(),
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: Vec::new(),
    });
    state.ingest_plan(
        21,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300, 600],
            requested_pages: vec![500],
            dirty_requested_pages: vec![500],
            evictable_pages: vec![200, 300, 600],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 500,
            size_bytes: 2_048,
            generation: 21,
            frontier_rank: 0,
            assigned_slot: Some(0),
            recycled_page_id: Some(300),
        }],
        "expected feedback-side completion to carry confirmed hot-frontier truth into the reconnected ancestor so a later recycle plan still preserves that ancestor even after the hotter descendant has already left residency"
    );
}

#[test]
fn virtual_geometry_runtime_state_drops_recent_hot_frontier_truth_when_page_leaves_live_extract_before_reappearing(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract_a = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(50, 500, None),
        ],
        pages: vec![
            page(200, true, 2_048),
            page(300, true, 2_048),
            page(500, false, 2_048),
        ],
    };
    let extract_b = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![render_cluster(30, 300, None), render_cluster(50, 500, None)],
        pages: vec![page(300, true, 2_048), page(500, false, 2_048)],
    };
    let extract_c = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(50, 500, None),
        ],
        pages: vec![
            page(200, true, 2_048),
            page(300, true, 2_048),
            page(500, false, 2_048),
        ],
    };

    state.register_extract(Some(&extract_a));
    state.ingest_plan(
        22,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: Vec::new(),
            dirty_requested_pages: Vec::new(),
            evictable_pages: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![20],
        requested_pages: Vec::new(),
        evictable_pages: Vec::new(),
        hot_resident_pages: vec![200],
    });
    state.ingest_plan(
        23,
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
        hot_resident_pages: Vec::new(),
    });

    state.register_extract(Some(&extract_b));
    state.register_extract(Some(&extract_c));
    state.ingest_plan(
        24,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: vec![500],
            dirty_requested_pages: vec![500],
            evictable_pages: vec![200, 300],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 500,
            size_bytes: 2_048,
            generation: 24,
            frontier_rank: 0,
            assigned_slot: Some(0),
            recycled_page_id: Some(200),
        }],
        "expected extract registration to clear stale recent hot-frontier truth when a page leaves the live extract, so a later reintroduced page does not keep an old hot bias across that extraction gap"
    );
}

#[test]
fn virtual_geometry_runtime_state_does_not_let_evicted_hot_descendant_bias_later_gpu_completion_frontier_merge(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(50, 500, None),
            render_cluster(70, 700, None),
            render_cluster(80, 800, Some(20)),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 2_048),
            page(500, false, 2_048),
            page(700, false, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        25,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 800],
            requested_pages: vec![500, 200],
            dirty_requested_pages: vec![500, 200],
            evictable_pages: vec![800, 300],
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![500, 200],
        evictable_pages: vec![800, 300],
        hot_resident_pages: vec![800],
    });
    state.complete_gpu_uploads_with_replacements(
        [(500, 1), (200, 0)],
        std::iter::empty(),
        &[800, 300],
    );
    state.apply_gpu_page_table_entries(&[(200, 0), (500, 1)]);
    state.ingest_plan(
        26,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![700],
            dirty_requested_pages: vec![700],
            evictable_pages: vec![200, 500],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 700,
            size_bytes: 2_048,
            generation: 26,
            frontier_rank: 0,
            assigned_slot: Some(0),
            recycled_page_id: Some(200),
        }],
        "expected runtime merge to drop hot-frontier truth as soon as the descendant page is evicted, so a later GPU-completed ancestor does not keep a stale hot bias from a page that no longer survives the still-live residency merge"
    );
}

#[test]
fn virtual_geometry_runtime_state_does_not_let_later_batch_eviction_keep_hot_descendant_bias_on_earlier_completion(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 5,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(50, 500, None),
            render_cluster(70, 700, None),
            render_cluster(80, 800, Some(20)),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 2_048),
            page(500, false, 2_048),
            page(700, false, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        27,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 800],
            requested_pages: vec![200, 500],
            dirty_requested_pages: vec![200, 500],
            evictable_pages: vec![800, 300],
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![200, 500],
        evictable_pages: vec![800, 300],
        hot_resident_pages: vec![800],
    });
    state.complete_gpu_uploads_with_replacements(
        [(200, 0), (500, 1)],
        std::iter::empty(),
        &[800, 300],
    );
    state.apply_gpu_page_table_entries(&[(200, 0), (500, 1)]);
    state.ingest_plan(
        28,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 500],
            requested_pages: vec![700],
            dirty_requested_pages: vec![700],
            evictable_pages: vec![200, 500],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 700,
            size_bytes: 2_048,
            generation: 28,
            frontier_rank: 0,
            assigned_slot: Some(0),
            recycled_page_id: Some(200),
        }],
        "expected the runtime completion batch to ignore hot descendants that are already claimed by later slot displacements in the same batch, so earlier ancestor completions do not keep a stale hot bias after the descendant disappears from the surviving residency truth"
    );
}

#[test]
fn virtual_geometry_runtime_state_does_not_let_removed_hot_descendant_bias_page_table_reconnect_frontier_merge(
) {
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(70, 700, None),
            render_cluster(80, 800, Some(20)),
        ],
        pages: vec![
            page(200, false, 2_048),
            page(300, true, 2_048),
            page(700, false, 2_048),
            page(800, true, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        27,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![300, 800],
            requested_pages: vec![200],
            dirty_requested_pages: vec![200],
            evictable_pages: vec![800, 300],
        },
    );
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![80],
        requested_pages: vec![200],
        evictable_pages: vec![800, 300],
        hot_resident_pages: vec![800],
    });
    state.apply_gpu_page_table_entries(&[(300, 0), (200, 2)]);
    state.ingest_plan(
        28,
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
            size_bytes: 2_048,
            generation: 28,
            frontier_rank: 0,
            assigned_slot: Some(2),
            recycled_page_id: Some(200),
        }],
        "expected final page-table apply to drop hot-frontier truth from descendants that no longer survive the confirmed table, so a later reconnect does not keep a stale hot bias from a page that has already disappeared from the authoritative resident set"
    );
}

#[test]
fn virtual_geometry_runtime_state_keeps_reassigned_page_table_owner_in_next_frontier_recycle_plan()
{
    let mut state = VirtualGeometryRuntimeState::default();
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            render_cluster(20, 200, None),
            render_cluster(30, 300, None),
            render_cluster(50, 500, None),
            render_cluster(70, 700, None),
        ],
        pages: vec![
            page(200, true, 2_048),
            page(300, true, 2_048),
            page(500, false, 2_048),
            page(700, false, 2_048),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        14,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300],
            requested_pages: vec![700],
            dirty_requested_pages: vec![700],
            evictable_pages: vec![200, 300],
        },
    );
    state.complete_gpu_uploads_with_replacements([(700, 1)], std::iter::empty(), &[200, 300]);
    state.apply_gpu_page_table_entries(&[(200, 0), (300, 1), (700, 1), (300, 2)]);
    state.refresh_hot_resident_pages(&VisibilityVirtualGeometryFeedback {
        visible_cluster_ids: vec![70],
        requested_pages: vec![500],
        evictable_pages: vec![300, 700],
        hot_resident_pages: vec![700],
    });
    state.ingest_plan(
        15,
        &VisibilityVirtualGeometryPageUploadPlan {
            resident_pages: vec![200, 300, 700],
            requested_pages: vec![500],
            dirty_requested_pages: vec![500],
            evictable_pages: vec![300, 700],
        },
    );

    let prepare = state.build_prepare_frame(&[]);

    assert_eq!(state.page_slot(300), Some(2));
    assert_eq!(state.page_slot(700), Some(1));
    assert_eq!(
        prepare.pending_page_requests,
        vec![VirtualGeometryPrepareRequest {
            page_id: 500,
            size_bytes: 2_048,
            generation: 15,
            frontier_rank: 0,
            assigned_slot: Some(2),
            recycled_page_id: Some(300),
        }],
        "expected page-table-confirmed slot reassignment to keep the moved resident page in runtime ownership so the next frontier recycle plan still evicts the colder moved page instead of the hot newly completed page"
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
        bounds_center: crate::core::math::Vec3::ZERO,
        bounds_radius: 1.0,
        screen_space_error: 1.0,
    }
}
