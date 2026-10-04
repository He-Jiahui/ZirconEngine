use std::time::Instant;

use super::*;
use crate::scene::viewport::pointer::{
    precision::PrecisionShape, viewport_pointer_route::ViewportPointerRoute,
};
use zircon_runtime_interface::math::Vec2;

#[test]
fn stable_candidate_topology_patches_geometry_without_replacing_nodes() {
    let mut router = ViewportOverlayPointerRouter::new();
    let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
    let initial = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(10.0, 20.0, 30.0, 40.0),
        100,
    );
    router.rebuild_surface_from_scratch(viewport_frame, &[initial]);
    let root_address = router
        .surface
        .tree
        .node(ROOT_NODE_ID)
        .expect("root must exist") as *const UiTreeNode;
    let candidate_address = router
        .surface
        .tree
        .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
        .expect("candidate must exist") as *const UiTreeNode;

    let next_frame = UiFrame::new(50.0, 60.0, 70.0, 80.0);
    let next = candidate_entry(FIRST_CANDIDATE_NODE_ID, next_frame, 300);
    assert!(try_apply_surface_delta(
        &mut router,
        viewport_frame,
        &[next]
    ));

    assert_eq!(
        router
            .surface
            .tree
            .node(ROOT_NODE_ID)
            .expect("root must remain") as *const UiTreeNode,
        root_address,
    );
    let candidate = router
        .surface
        .tree
        .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
        .expect("candidate must remain");
    assert_eq!(candidate as *const UiTreeNode, candidate_address);
    assert_eq!(candidate.layout_cache.frame, next_frame);
    assert_eq!(candidate.z_index, 300);
}

#[test]
fn optimization_batch_r6_wave7_editor644_geometry_changes_reserve_candidate_bound() {
    let source = include_str!("../rebuild_surface.rs");
    assert!(source
        .contains("let mut changes = Vec::with_capacity(candidates.len().saturating_add(2));"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave7_editor644_geometry_changes_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const CANDIDATES_PER_SAMPLE: usize = 65_536;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(editor644_measure_geometry_changes(
                CANDIDATES_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(editor644_measure_geometry_changes(
                CANDIDATES_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(editor644_measure_geometry_changes(
                CANDIDATES_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(editor644_measure_geometry_changes(
                CANDIDATES_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = editor644_p95(&legacy_samples);
    let optimized_p95 = editor644_p95(&optimized_samples);
    println!(
        "EDITOR644_PREALLOCATED_OVERLAY_GEOMETRY_CHANGES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} candidates_per_sample={CANDIDATES_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated overlay geometry changes must be at least 15% faster at P95"
    );
}

fn editor644_measure_geometry_changes(candidate_count: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut changes = if optimized {
        Vec::with_capacity(candidate_count.saturating_add(2))
    } else {
        Vec::new()
    };
    for candidate_index in 0..candidate_count {
        changes.push(candidate_index);
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    std::hint::black_box(changes);
    elapsed
}

fn editor644_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

#[test]
fn stable_candidate_frame_uses_exact_runtime_geometry_publication() {
    let mut router = ViewportOverlayPointerRouter::new();
    let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
    let initial = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(10.0, 20.0, 30.0, 40.0),
        100,
    );
    router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

    let next = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(50.0, 60.0, 30.0, 40.0),
        100,
    );
    assert!(try_apply_surface_delta(
        &mut router,
        viewport_frame,
        &[next]
    ));

    assert_eq!(
        router
            .surface
            .last_rebuild_report
            .arranged_outer_node_visit_count,
        1
    );
    assert_eq!(
        router
            .surface
            .last_rebuild_report
            .hit_grid_outer_node_visit_count,
        1
    );
    assert_eq!(
        router
            .surface
            .last_rebuild_report
            .render_outer_node_visit_count,
        1
    );
}

#[test]
fn changed_candidate_count_rejects_patch_before_geometry_mutation() {
    let mut router = ViewportOverlayPointerRouter::new();
    let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
    let initial_frame = UiFrame::new(10.0, 20.0, 30.0, 40.0);
    let initial = candidate_entry(FIRST_CANDIDATE_NODE_ID, initial_frame, 100);
    router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

    let next = [
        candidate_entry(
            FIRST_CANDIDATE_NODE_ID,
            UiFrame::new(50.0, 60.0, 70.0, 80.0),
            300,
        ),
        candidate_entry(
            FIRST_CANDIDATE_NODE_ID + 1,
            UiFrame::new(100.0, 120.0, 70.0, 80.0),
            300,
        ),
    ];
    assert!(!try_apply_surface_delta(&mut router, viewport_frame, &next));
    let candidate = router
        .surface
        .tree
        .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
        .expect("rejected patch must preserve the existing candidate");
    assert_eq!(candidate.layout_cache.frame, initial_frame);
    assert_eq!(candidate.z_index, 100);
}

#[test]
fn changed_candidate_id_rejects_patch_before_geometry_mutation() {
    let mut router = ViewportOverlayPointerRouter::new();
    let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
    let initial_frame = UiFrame::new(10.0, 20.0, 30.0, 40.0);
    let initial = candidate_entry(FIRST_CANDIDATE_NODE_ID, initial_frame, 100);
    router.rebuild_surface_from_scratch(viewport_frame, &[initial]);

    let next = candidate_entry(
        FIRST_CANDIDATE_NODE_ID + 1,
        UiFrame::new(50.0, 60.0, 70.0, 80.0),
        300,
    );
    assert!(!try_apply_surface_delta(
        &mut router,
        viewport_frame,
        &[next]
    ));
    let candidate = router
        .surface
        .tree
        .node(UiNodeId::new(FIRST_CANDIDATE_NODE_ID))
        .expect("rejected patch must preserve the existing candidate");
    assert_eq!(candidate.layout_cache.frame, initial_frame);
    assert_eq!(candidate.z_index, 100);
}

#[test]
fn changed_candidate_route_releases_pointer_capture_before_reuse() {
    let mut router = ViewportOverlayPointerRouter::new();
    let viewport_frame = UiFrame::new(0.0, 0.0, 640.0, 480.0);
    let candidate_id = UiNodeId::new(FIRST_CANDIDATE_NODE_ID);
    let initial = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(10.0, 20.0, 30.0, 40.0),
        100,
    );
    router.rebuild_surface_from_scratch(viewport_frame, &[initial]);
    lock_shared_resolution_state(router.shared.as_ref())
        .candidates
        .insert(
            candidate_id,
            PrecisionCandidate {
                route: ViewportPointerRoute::Renderable {
                    owner: FIRST_CANDIDATE_NODE_ID,
                },
                priority: 0,
                shape: PrecisionShape::Circle {
                    center: Vec2::new(10.0, 20.0),
                    radius_px: 1.0,
                    threshold_px: 0.0,
                    depth: 0.0,
                },
            },
        );
    router
        .surface
        .capture_pointer(candidate_id)
        .expect("candidate must be a valid pointer-capture owner");

    let mut next = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(50.0, 60.0, 70.0, 80.0),
        300,
    );
    next.candidate.route = ViewportPointerRoute::Renderable { owner: 999 };
    assert!(try_apply_surface_delta(
        &mut router,
        viewport_frame,
        &[next]
    ));
    assert_eq!(router.surface.focus.captured, None);
}

#[test]
fn stable_candidate_keys_patch_shared_map_values_without_reallocating_nodes() {
    let candidate_id = UiNodeId::new(FIRST_CANDIDATE_NODE_ID);
    let mut shared = crate::scene::viewport::pointer::precision::SharedResolutionState::default();
    let initial = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(10.0, 20.0, 30.0, 40.0),
        100,
    );
    shared
        .candidates
        .insert(candidate_id, initial.candidate.clone());
    let initial_address = shared
        .candidates
        .get(&candidate_id)
        .expect("initial candidate must exist")
        as *const PrecisionCandidate;

    let mut next = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(50.0, 60.0, 70.0, 80.0),
        300,
    );
    next.candidate.route = ViewportPointerRoute::Renderable { owner: 999 };
    assert!(publish_retained_candidates(&mut shared, vec![next], true));

    let current = shared
        .candidates
        .get(&candidate_id)
        .expect("patched candidate must exist");
    assert_eq!(current as *const PrecisionCandidate, initial_address);
    assert_eq!(
        current.route,
        ViewportPointerRoute::Renderable { owner: 999 }
    );
}

#[test]
fn changed_candidate_keys_rebuild_shared_map_before_value_publication() {
    let mut shared = crate::scene::viewport::pointer::precision::SharedResolutionState::default();
    let initial = candidate_entry(
        FIRST_CANDIDATE_NODE_ID,
        UiFrame::new(10.0, 20.0, 30.0, 40.0),
        100,
    );
    shared.candidates.insert(initial.node_id, initial.candidate);

    let next = candidate_entry(
        FIRST_CANDIDATE_NODE_ID + 1,
        UiFrame::new(50.0, 60.0, 70.0, 80.0),
        300,
    );
    assert!(!publish_retained_candidates(&mut shared, vec![next], false));
    assert!(!shared
        .candidates
        .contains_key(&UiNodeId::new(FIRST_CANDIDATE_NODE_ID)));
    assert!(shared
        .candidates
        .contains_key(&UiNodeId::new(FIRST_CANDIDATE_NODE_ID + 1)));
}

fn try_apply_surface_delta(
    router: &mut ViewportOverlayPointerRouter,
    viewport_frame: UiFrame,
    candidates: &[RetainedCandidateEntry],
) -> bool {
    let classification = router.classify_surface_delta(viewport_frame, candidates);
    if matches!(&classification.delta, ViewportOverlaySurfaceDelta::Topology) {
        return false;
    }
    router.apply_surface_delta(viewport_frame, candidates, classification.delta);
    true
}

fn candidate_entry(node_id: u64, frame: UiFrame, z_index: i32) -> RetainedCandidateEntry {
    RetainedCandidateEntry {
        node_id: UiNodeId::new(node_id),
        path_suffix: node_id.saturating_add(1),
        frame,
        z_index,
        candidate: PrecisionCandidate {
            route: ViewportPointerRoute::Renderable { owner: node_id },
            priority: 0,
            shape: PrecisionShape::Circle {
                center: Vec2::new(frame.x, frame.y),
                radius_px: 1.0,
                threshold_px: 0.0,
                depth: 0.0,
            },
        },
    }
}
