use std::collections::BTreeMap;

use zircon_runtime::{
    core::framework::picking::{
        HitTarget, PickingAxis, PickingDebugMetricKind, PointerAction, PointerButton, PointerId,
        PointerScrollUnit,
    },
    ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt},
};
use zircon_runtime_interface::{
    math::Vec2,
    ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{UiFrame, UiPoint},
        tree::{UiInputPolicy, UiTreeNode},
    },
};

use crate::scene::viewport::GizmoAxis;

use super::{
    candidates::{candidate_z_index, interactive_state_flags, passive_state_flags},
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
}

#[test]
fn overlay_router_debug_feed_reports_runtime_picking_route_at_point() {
    let mut router = ViewportOverlayPointerRouter::new();
    seed_debug_feed_router(&mut router);

    let point = UiPoint::new(50.0, 50.0);
    let feed = router
        .debug_feed_at(point)
        .expect("debug feed should be readable");
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
    let runtime_input = dispatch
        .runtime_input
        .expect("dispatch should carry the runtime pointer input");
    assert_eq!(runtime_input.pointer(), PointerId::new(1));
    assert_eq!(runtime_input.location.position, Vec2::new(50.0, 50.0));
    assert_eq!(
        runtime_input.action,
        PointerAction::Move { delta: Vec2::ZERO }
    );
}

#[test]
fn overlay_router_dispatch_maps_release_and_scroll_through_runtime_pointer_input() {
    let mut router = ViewportOverlayPointerRouter::new();
    seed_debug_feed_router(&mut router);

    let down = router
        .handle_down(UiPoint::new(50.0, 50.0))
        .expect("down should dispatch")
        .runtime_input
        .expect("down dispatch should carry runtime input");
    assert_eq!(down.action, PointerAction::Press(PointerButton::Primary));

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
    assert_eq!(
        up.runtime_input
            .expect("up dispatch should carry runtime input")
            .action,
        PointerAction::Release(PointerButton::Primary)
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
    assert_eq!(
        scroll
            .runtime_input
            .expect("scroll dispatch should carry runtime input")
            .action,
        PointerAction::Scroll {
            unit: PointerScrollUnit::Pixel,
            delta: Vec2::new(0.0, -8.0),
        }
    );
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
        last_route: None,
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
