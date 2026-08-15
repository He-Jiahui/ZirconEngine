use crate::core::editor_event::EditorEvent;
use crate::core::editor_event::SelectionHostEvent;
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_hierarchy_pointer_click, dispatch_shared_welcome_recent_pointer_click,
    BuiltinWelcomeSurfaceTemplateBridge,
};
use crate::ui::retained_host::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerRoute, HierarchyPointerState,
};
use crate::ui::retained_host::welcome_recent_geometry::{
    welcome_recent_row_geometry, welcome_recent_viewport,
};
use crate::ui::retained_host::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
    WelcomeRecentPointerRoute, WelcomeRecentPointerState,
};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_welcome_recent_pointer_bridge_scrolls_and_dispatches_remove_action() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWelcomeSurfaceTemplateBridge::new().expect("builtin welcome bridge should build");
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(welcome_layout(12), WelcomeRecentPointerState::default());

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(120.0, 190.0), 140.0)
        .expect("welcome recent list should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(welcome_layout(12), scrolled.state.clone());
    let item_index = 3usize;
    let remove = welcome_recent_row_geometry(
        welcome_recent_viewport(UiSize::new(720.0, 620.0)),
        item_index,
        scrolled.state.scroll_offset,
    )
    .remove;
    let dispatched = dispatch_shared_welcome_recent_pointer_click(
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(
            remove.x + remove.width * 0.5,
            remove.y + remove.height * 0.5,
        ),
    )
    .expect("shared welcome pointer route should dispatch remove recent project");
    assert_eq!(
        dispatched.pointer.route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Remove,
            path: "E:/Projects/demo-03".to_string(),
        })
    );
    assert_eq!(
        dispatched.event,
        Some(WelcomeHostEvent::RemoveRecentProject {
            path: "E:/Projects/demo-03".to_string(),
        })
    );
}

#[test]
fn shared_welcome_recent_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state() {
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    let layout = welcome_layout(8);
    let state = WelcomeRecentPointerState::default();

    assert!(pointer_bridge.sync(layout.clone(), state.clone()));
    assert!(!pointer_bridge.sync(layout, state));
}

#[test]
fn welcome_recent_pointer_routes_large_lists_without_materializing_rows_or_replacing_authority() {
    let layout = welcome_layout(10_000);
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    let initial_authority_generation = pointer_bridge.surface_authority_generation_for_test();
    assert!(pointer_bridge.sync(layout.clone(), WelcomeRecentPointerState::default()));

    assert_eq!(
        pointer_bridge.surface_authority_generation_for_test(),
        initial_authority_generation,
        "pane geometry should patch the two-node surface without replacing authority"
    );
    assert_eq!(
        pointer_bridge.surface_node_count_for_test(),
        2,
        "logical recent rows and their actions must not become retained hit-test nodes"
    );

    let authority_generation = pointer_bridge.surface_authority_generation_for_test();
    let list_point = UiPoint::new(120.0, 190.0);
    let scrolled = pointer_bridge
        .handle_scroll(list_point, 310_000.0)
        .expect("large recent list should accept a bounded scroll");
    let item_index = scrolled
        .state
        .hovered_item_index
        .expect("post-scroll pointer position should resolve an arithmetic row");
    assert!(item_index > 1_000);
    assert_eq!(scrolled.route, Some(WelcomeRecentPointerRoute::ListSurface));
    assert_eq!(
        pointer_bridge.surface_authority_generation_for_test(),
        authority_generation,
        "scrolling must preserve the surface, dispatcher, and route authority"
    );
    assert_eq!(pointer_bridge.surface_node_count_for_test(), 2);

    let viewport = welcome_recent_viewport(layout.pane_size);
    let geometry = welcome_recent_row_geometry(viewport, item_index, scrolled.state.scroll_offset);
    let remove_point = UiPoint::new(
        geometry.remove.x + geometry.remove.width * 0.5,
        geometry.remove.y + geometry.remove.height * 0.5,
    );
    let moved = pointer_bridge
        .handle_move(remove_point)
        .expect("remove hover should use the same arithmetic row projection");
    assert_eq!(
        moved.route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Remove,
            path: layout.recent_project_paths[item_index].clone(),
        })
    );
    let clicked = pointer_bridge
        .handle_click(remove_point)
        .expect("remove click should use the same arithmetic row projection");
    assert_eq!(clicked.route, moved.route);

    let open_point = UiPoint::new(
        geometry.open.x + geometry.open.width * 0.5,
        geometry.open.y + geometry.open.height * 0.5,
    );
    assert_eq!(
        pointer_bridge
            .handle_click(open_point)
            .expect("open click should project from the same row geometry")
            .route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Open,
            path: layout.recent_project_paths[item_index].clone(),
        })
    );

    let next_geometry =
        welcome_recent_row_geometry(viewport, item_index + 1, scrolled.state.scroll_offset);
    let row_gap_point = UiPoint::new(
        geometry.row.x + geometry.row.width * 0.5,
        (geometry.row.bottom() + next_geometry.row.y) * 0.5,
    );
    let gap_move = pointer_bridge
        .handle_move(row_gap_point)
        .expect("row gaps should remain part of the list surface");
    assert_eq!(gap_move.route, Some(WelcomeRecentPointerRoute::ListSurface));
    assert_eq!(gap_move.state.hovered_item_index, None);
    assert_eq!(gap_move.state.hovered_action, None);

    pointer_bridge
        .handle_move(remove_point)
        .expect("remove hover should restore the row state before scrolling the gap");
    let gap_scroll = pointer_bridge
        .handle_scroll(row_gap_point, 0.0)
        .expect("scrolling over a row gap should clear stale row hover");
    assert_eq!(
        gap_scroll.route,
        Some(WelcomeRecentPointerRoute::ListSurface)
    );
    assert_eq!(gap_scroll.state.hovered_item_index, None);
    assert_eq!(gap_scroll.state.hovered_action, None);
}

#[test]
fn shared_hierarchy_pointer_bridge_scrolls_and_dispatches_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_hierarchy_pointer");
    let nodes = harness
        .runtime
        .editor_snapshot()
        .scene_entries
        .iter()
        .map(|entry| entry.id.to_string())
        .collect::<Vec<_>>();
    assert!(
        nodes.len() >= 2,
        "default fixture should expose hierarchy rows"
    );

    let mut pointer_bridge = HierarchyPointerBridge::new();
    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 40.0,
            node_ids: nodes.clone(),
        },
        HierarchyPointerState::default(),
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(120.0, 20.0), 24.0)
        .expect("hierarchy list should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 40.0,
            node_ids: nodes.clone(),
        },
        scrolled.state.clone(),
    );
    let dispatched = dispatch_shared_hierarchy_pointer_click(
        &harness.runtime,
        &mut pointer_bridge,
        UiPoint::new(80.0, 28.0),
    )
    .expect("shared hierarchy pointer route should dispatch scene-node selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(HierarchyPointerRoute::Node {
            item_index: 1,
            node_id: nodes[1].clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("hierarchy node click should dispatch into runtime");
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
            node_id: nodes[1].parse().unwrap(),
        })
    );
}

#[test]
fn shared_hierarchy_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state() {
    let mut pointer_bridge = HierarchyPointerBridge::new();
    let layout = HierarchyPointerLayout {
        pane_width: 260.0,
        pane_height: 40.0,
        node_ids: vec!["1".to_string(), "2".to_string()],
    };
    let state = HierarchyPointerState::default();

    assert!(pointer_bridge.sync(layout.clone(), state.clone()));
    assert!(!pointer_bridge.sync(layout, state));
}

#[test]
fn hierarchy_pointer_routes_large_lists_without_materializing_rows_or_replacing_authority() {
    let nodes = (0..10_000)
        .map(|index| format!("node-{index}"))
        .collect::<Vec<_>>();
    let mut pointer_bridge = HierarchyPointerBridge::new();
    let initial_authority_generation = pointer_bridge.surface_authority_generation_for_test();
    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 120.0,
            node_ids: nodes.clone(),
        },
        HierarchyPointerState::default(),
    );

    assert_eq!(
        pointer_bridge.surface_authority_generation_for_test(),
        initial_authority_generation,
        "pane geometry should patch the two-node surface without replacing authority"
    );
    assert_eq!(
        pointer_bridge.surface_node_count_for_test(),
        2,
        "logical hierarchy rows must not become retained hit-test nodes"
    );
    let authority_generation = pointer_bridge.surface_authority_generation_for_test();
    let point = UiPoint::new(80.0, 20.0);
    let scrolled = pointer_bridge
        .handle_scroll(point, 135_000.0)
        .expect("large hierarchy should accept a bounded scroll");
    let Some(HierarchyPointerRoute::Node {
        item_index,
        node_id,
    }) = scrolled.route
    else {
        panic!("the post-scroll pointer position should resolve an arithmetic row route");
    };
    assert!(item_index > 1_000);
    assert_eq!(node_id, nodes[item_index]);
    assert_eq!(
        pointer_bridge.surface_authority_generation_for_test(),
        authority_generation,
        "scrolling must preserve the constant-size surface, dispatcher, and route authority"
    );
    assert_eq!(pointer_bridge.surface_node_count_for_test(), 2);

    let clicked = pointer_bridge
        .handle_click(point)
        .expect("post-scroll click should use the same arithmetic row projection");
    assert_eq!(
        clicked.route,
        Some(HierarchyPointerRoute::Node {
            item_index,
            node_id: nodes[item_index].clone(),
        })
    );
    assert_eq!(
        pointer_bridge
            .handle_move(UiPoint::new(2.0, point.y))
            .expect("row inset should remain part of the list surface")
            .route,
        Some(HierarchyPointerRoute::ListSurface)
    );
}

fn welcome_layout(count: usize) -> WelcomeRecentPointerLayout {
    WelcomeRecentPointerLayout {
        pane_size: UiSize::new(720.0, 620.0),
        recent_project_paths: (0..count)
            .map(|index| format!("E:/Projects/demo-{index:02}"))
            .collect(),
    }
}
