use std::{cell::Cell, collections::BTreeMap, sync::Arc};

use zircon_runtime::{
    core::framework::picking::{HitTarget, PickingAxis, PickingDebugMetricKind},
    core::framework::render::{SceneGizmoKind, SceneGizmoOverlayExtract},
    scene::Scene,
    ui::surface::UiSurface,
};
use zircon_runtime_interface::{
    math::Vec2,
    ui::{
        dispatch::UiPointerEvent,
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{UiFrame, UiPoint},
        surface::UiPointerEventKind,
        tree::{UiInputPolicy, UiTreeError, UiTreeNode},
    },
};

use crate::scene::viewport::{
    render_packet::build_render_packet, GizmoAxis, SceneViewportSettings, ViewportCameraSnapshot,
    ViewportInteractionExtractCache, ViewportInteractionExtractPointerResolution,
};

use super::{
    candidates::{
        candidate_z_index, interactive_state_flags, passive_state_flags, renderable_candidates,
    },
    constants::{
        GIZMO_PRIORITY, HANDLE_PRIORITY, RENDERABLE_PRIORITY, ROOT_NODE_ID, VIEWPORT_NODE_ID,
    },
    overlay_router::ViewportOverlayPointerRouter,
    precision::{PrecisionCandidate, PrecisionShape, SharedResolutionState},
    runtime_picking_adapter::resolve_runtime_route,
    viewport_pointer_route::ViewportPointerRoute,
};

const TEST_HANDLE_NODE_ID: UiNodeId = UiNodeId::new(128);
const TEST_RENDERABLE_NODE_ID: UiNodeId = UiNodeId::new(129);
const TEST_GIZMO_NODE_ID: UiNodeId = UiNodeId::new(130);

#[test]
fn pointer_module_keeps_runtime_route_adapter_as_authoritative_resolver() {
    let pointer_mod = include_str!("mod.rs");
    let overlay_router_mod = include_str!("overlay_router/mod.rs");
    let dispatcher = include_str!("overlay_router/build_dispatcher.rs");

    assert!(
        !pointer_mod.contains("\nfn "),
        "pointer/mod.rs should stay structural; put behavior in child modules"
    );
    assert!(
        !pointer_mod.contains("mod tests {"),
        "pointer/mod.rs should not regain inline test bodies"
    );

    for forbidden in ["better_score", "resolve_best_route"] {
        assert!(
            !overlay_router_mod.contains(forbidden) && !dispatcher.contains(forbidden),
            "editor pointer routing must not reintroduce `{forbidden}`; use runtime picking adapter"
        );
    }
    assert!(
        dispatcher.contains("resolve_runtime_route"),
        "viewport overlay dispatcher should resolve through runtime picking"
    );
    let event = include_str!("overlay_router/viewport_overlay_pointer_router_event.rs");
    assert!(
        !event.contains("self.debug_feed_at(point)"),
        "pointer dispatch must reuse the route pass for its debug feed"
    );
    let hit_frames = include_str!("precision/precision_shape_hit_frame.rs");
    assert!(
        !hit_frames.contains("let points: Vec<_>"),
        "ring hit frames must scan endpoints without allocating a temporary Vec"
    );
}

#[test]
fn overlay_router_debug_feed_reports_runtime_picking_route_at_point() {
    let mut router = ViewportOverlayPointerRouter::new();
    seed_debug_feed_router(&mut router);

    let point = UiPoint::new(50.0, 50.0);
    let feed = router.debug_feed_at(point);
    let row = feed
        .pointers
        .first()
        .expect("overlapping candidates should produce a pointer row");

    assert_eq!(feed.metric(PickingDebugMetricKind::RawHits), Some(2));
    assert_eq!(feed.metric(PickingDebugMetricKind::BackendOutputs), Some(1));
    assert_eq!(feed.metric(PickingDebugMetricKind::HoveredHits), Some(1));
    assert_eq!(row.backend_output_count, 1);
    assert_eq!(
        row.top_target,
        Some(HitTarget::handle_axis(7, PickingAxis::X))
    );
    let dispatch = router.handle_move(point).expect("route should dispatch");

    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
    assert_eq!(dispatch.picking_debug_feed, Some(feed));
}

#[test]
fn viewport_pointer_surface_errors_remain_typed() {
    let mut router = ViewportOverlayPointerRouter::new();
    let removed = router.surface.tree.nodes.remove(&ROOT_NODE_ID);
    assert!(
        removed.is_some(),
        "test fixture must remove the retained root"
    );

    let error = router
        .handle_move(UiPoint::new(50.0, 50.0))
        .expect_err("a retained root missing from the tree must fail routing");

    assert_eq!(error, UiTreeError::MissingNode(ROOT_NODE_ID));
}

#[test]
fn poisoned_shared_resolution_state_recovers_without_dropping_pointer_route() {
    let mut router = ViewportOverlayPointerRouter::new();
    seed_debug_feed_router(&mut router);

    let shared = Arc::clone(&router.shared);
    let poisoned = std::thread::spawn(move || {
        let _state = shared
            .lock()
            .expect("test fixture should acquire the shared state");
        panic!("test fixture poisons the shared resolution state");
    })
    .join();
    assert!(poisoned.is_err());

    let dispatch = router
        .handle_move(UiPoint::new(50.0, 50.0))
        .expect("the pointer router must recover its shared state after a poisoned lock");
    assert_eq!(
        dispatch.route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
}

#[test]
fn overlay_router_dispatch_keeps_route_and_debug_feed_for_release_and_scroll() {
    let mut router = ViewportOverlayPointerRouter::new();
    seed_debug_feed_router(&mut router);

    router
        .handle_down(UiPoint::new(50.0, 50.0))
        .expect("down should dispatch");

    let up = router
        .handle_up(UiPoint::new(50.0, 50.0))
        .expect("up should dispatch");
    assert_eq!(
        up.route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
    assert_eq!(
        up.picking_debug_feed
            .as_ref()
            .and_then(|feed| feed.metric(PickingDebugMetricKind::HoveredHits)),
        Some(1)
    );
    let scroll = router
        .handle_scroll(UiPoint::new(50.0, 50.0), -8.0)
        .expect("scroll should dispatch");
    assert_eq!(
        scroll.route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
    assert_eq!(
        scroll
            .picking_debug_feed
            .as_ref()
            .and_then(|feed| feed.metric(PickingDebugMetricKind::HoveredHits)),
        Some(1)
    );
}

#[test]
fn viewport_pointer_dispatch_does_not_expose_synthetic_runtime_input() {
    let dispatch = include_str!("viewport_pointer_dispatch.rs");
    let adapter = include_str!("runtime_picking_adapter.rs");
    let event = include_str!("overlay_router/viewport_overlay_pointer_router_event.rs");

    assert!(!dispatch.contains("PointerInput"));
    assert!(!dispatch.contains("runtime_input"));
    assert!(!adapter.contains("EDITOR_VIEWPORT_HANDLE"));
    assert!(!adapter.contains("runtime_pointer_input_for_event"));
    assert!(!event.contains("runtime_pointer_input_for_event"));
}

#[test]
fn stable_scene_sync_skips_candidate_and_handle_rebuilds() {
    let scene = Scene::new();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = zircon_runtime_interface::math::UVec2::new(1280, 720);
    let handle_builds = Cell::new(0);
    let additional_gizmo_builds = Cell::new(0);
    let cache = ViewportInteractionExtractCache::default();
    let mut router = ViewportOverlayPointerRouter::new();

    let packet = build_render_packet(&scene, &settings, &camera, None, viewport);
    let first = cache.resolve_from_render_packet(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        &packet.scene.meshes,
        || {
            handle_builds.set(handle_builds.get() + 1);
            Vec::new()
        },
        || {
            additional_gizmo_builds.set(additional_gizmo_builds.get() + 1);
            vec![empty_scene_gizmo(808)]
        },
    );
    assert!(first.scene_gizmos().iter().any(|gizmo| gizmo.owner == 808));
    assert!(router.sync_scene(&camera, viewport, scene.world_generation(), first));
    let second = cache.resolve_for_pointer(&scene, None, &settings, &camera, viewport);
    let ViewportInteractionExtractPointerResolution::Ready(second) = second else {
        panic!("render publication must make the extract available to pointer routing");
    };
    assert!(!router.sync_scene(&camera, viewport, scene.world_generation(), second));

    assert_eq!(handle_builds.get(), 1);
    assert_eq!(additional_gizmo_builds.get(), 1);
}

#[test]
fn clearing_scene_removes_the_previous_pointer_product_before_next_render_publication() {
    let scene = Scene::new();
    let settings = SceneViewportSettings::default();
    let camera = ViewportCameraSnapshot::default();
    let viewport = zircon_runtime_interface::math::UVec2::new(1280, 720);
    let cache = ViewportInteractionExtractCache::default();
    let mut router = ViewportOverlayPointerRouter::new();
    let packet = build_render_packet(&scene, &settings, &camera, None, viewport);
    let extract = cache.resolve_from_render_packet(
        &scene,
        None,
        &settings,
        &camera,
        viewport,
        &packet.scene.meshes,
        Vec::new,
        Vec::new,
    );

    assert!(router.sync_scene(&camera, viewport, scene.world_generation(), extract));
    assert!(router.clear_scene());
    assert_eq!(
        router
            .handle_move(UiPoint::new(50.0, 50.0))
            .expect("the empty pointer surface must remain dispatchable")
            .route,
        None
    );
}

fn empty_scene_gizmo(owner: u64) -> SceneGizmoOverlayExtract {
    SceneGizmoOverlayExtract {
        owner,
        kind: SceneGizmoKind::NavigationMesh,
        selected: false,
        lines: Vec::new(),
        wire_shapes: Vec::new(),
        icons: Vec::new(),
        pick_shapes: Vec::new(),
    }
}

#[test]
fn renderable_candidates_consume_runtime_meshes_and_collapse_primitives_by_owner() {
    let scene = Scene::new();
    let mut render_meshes = scene.to_render_extract().scene.meshes;
    let first = render_meshes
        .first()
        .cloned()
        .expect("default scene should expose one runtime render mesh");
    render_meshes.insert(0, first.clone());

    let candidates = renderable_candidates(&render_meshes);

    assert_eq!(
        candidates
            .iter()
            .filter(|candidate| candidate.owner == first.node_id)
            .count(),
        1
    );
    assert!(candidates
        .iter()
        .any(|candidate| candidate.owner == first.node_id));
}

#[test]
fn runtime_route_resolution_prefers_runtime_target_priority_over_stack_order() {
    let mut candidates = BTreeMap::new();
    candidates.insert(
        TEST_RENDERABLE_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::Renderable { owner: 3 },
            priority: RENDERABLE_PRIORITY,
            shape: debug_circle(0.0),
        },
    );
    candidates.insert(
        TEST_HANDLE_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::HandleAxis {
                owner: 7,
                axis: GizmoAxis::X,
            },
            priority: HANDLE_PRIORITY,
            shape: debug_circle(100.0),
        },
    );

    let route = resolve_runtime_route(
        &candidates,
        &[TEST_RENDERABLE_NODE_ID, TEST_HANDLE_NODE_ID],
        UiPoint::new(50.0, 50.0),
    );

    assert_eq!(
        route,
        Some(ViewportPointerRoute::HandleAxis {
            owner: 7,
            axis: GizmoAxis::X,
        })
    );
}

#[test]
fn runtime_route_resolution_prefers_scene_gizmo_over_renderable_depth() {
    let mut candidates = BTreeMap::new();
    candidates.insert(
        TEST_RENDERABLE_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::Renderable { owner: 3 },
            priority: RENDERABLE_PRIORITY,
            shape: debug_circle(0.0),
        },
    );
    candidates.insert(
        TEST_GIZMO_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::SceneGizmo { owner: 5 },
            priority: GIZMO_PRIORITY,
            shape: debug_circle(100.0),
        },
    );

    let route = resolve_runtime_route(
        &candidates,
        &[TEST_RENDERABLE_NODE_ID, TEST_GIZMO_NODE_ID],
        UiPoint::new(50.0, 50.0),
    );

    assert_eq!(route, Some(ViewportPointerRoute::SceneGizmo { owner: 5 }));
}

fn seed_debug_feed_router(router: &mut ViewportOverlayPointerRouter) {
    let mut surface = UiSurface::new(UiTreeId::new(
        "zircon.editor.viewport.pointer.debug-feed-test",
    ));
    let frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("debug.pointer.root"))
            .with_frame(frame)
            .with_state_flags(passive_state_flags())
            .with_input_policy(UiInputPolicy::Receive),
    );
    surface
        .tree
        .insert_child(
            ROOT_NODE_ID,
            UiTreeNode::new(VIEWPORT_NODE_ID, UiNodePath::new("debug.pointer.viewport"))
                .with_frame(frame)
                .with_state_flags(interactive_state_flags())
                .with_input_policy(UiInputPolicy::Receive),
        )
        .expect("root should exist");
    insert_debug_candidate_node(&mut surface, TEST_RENDERABLE_NODE_ID, RENDERABLE_PRIORITY);
    insert_debug_candidate_node(&mut surface, TEST_HANDLE_NODE_ID, HANDLE_PRIORITY);
    surface.rebuild();
    router.surface = surface;

    let mut candidates = BTreeMap::new();
    candidates.insert(
        TEST_RENDERABLE_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::Renderable { owner: 3 },
            priority: RENDERABLE_PRIORITY,
            shape: debug_circle(0.1),
        },
    );
    candidates.insert(
        TEST_HANDLE_NODE_ID,
        PrecisionCandidate {
            route: ViewportPointerRoute::HandleAxis {
                owner: 7,
                axis: GizmoAxis::X,
            },
            priority: HANDLE_PRIORITY,
            shape: debug_circle(10.0),
        },
    );
    *router
        .shared
        .lock()
        .expect("shared route state should be writable") = SharedResolutionState {
        candidates,
        renderer_visible_spatial_pick_source: None,
        last_route: None,
        last_debug_feed: None,
    };
}

fn insert_debug_candidate_node(surface: &mut UiSurface, node_id: UiNodeId, priority: u8) {
    surface
        .tree
        .insert_child(
            VIEWPORT_NODE_ID,
            UiTreeNode::new(
                node_id,
                UiNodePath::new(format!("debug.pointer.{node_id:?}")),
            )
            .with_frame(UiFrame::new(40.0, 40.0, 20.0, 20.0))
            .with_state_flags(interactive_state_flags())
            .with_input_policy(UiInputPolicy::Receive)
            .with_z_index(candidate_z_index(priority)),
        )
        .expect("viewport should exist");
}

fn debug_circle(depth: f32) -> PrecisionShape {
    PrecisionShape::Circle {
        center: Vec2::new(50.0, 50.0),
        radius_px: 8.0,
        threshold_px: 12.0,
        depth,
    }
}
