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
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint, UiSize};

#[test]
fn shared_welcome_recent_pointer_bridge_scrolls_and_dispatches_remove_action() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWelcomeSurfaceTemplateBridge::new().expect("builtin welcome bridge should build");
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(welcome_layout(12));

    let scrolled = pointer_bridge.handle_scroll(UiPoint::new(120.0, 190.0), 140.0);
    assert!(scrolled.state.scroll_offset > 0.0);
    assert!(scrolled.changed);

    pointer_bridge.sync(welcome_layout(12));
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
fn welcome_recent_pointer_routes_only_inside_the_published_list_viewport() {
    let _guard = env_lock().lock().unwrap();
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    let published_viewport = UiFrame::new(420.0, 120.0, 240.0, 300.0);
    let mut layout = welcome_layout(1);
    layout.viewport = published_viewport;
    pointer_bridge.sync(layout);

    let published_remove = welcome_recent_row_geometry(published_viewport, 0, 0.0)
        .remove
        .center();
    assert_eq!(
        pointer_bridge.handle_click(published_remove).route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index: 0,
            action: WelcomeRecentPointerAction::Remove,
        })
    );

    let stale_remove =
        welcome_recent_row_geometry(welcome_recent_viewport(UiSize::new(720.0, 620.0)), 0, 0.0)
            .remove
            .center();
    assert_eq!(pointer_bridge.handle_click(stale_remove).route, None);
}

#[test]
fn shared_welcome_recent_pointer_bridge_dispatches_recovery_action() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWelcomeSurfaceTemplateBridge::new().expect("builtin welcome bridge should build");
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(welcome_layout(2));
    let item_index = 1usize;
    let recover = welcome_recent_row_geometry(
        welcome_recent_viewport(UiSize::new(720.0, 620.0)),
        item_index,
        0.0,
    )
    .recover;
    let dispatched = dispatch_shared_welcome_recent_pointer_click(
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(
            recover.x + recover.width * 0.5,
            recover.y + recover.height * 0.5,
        ),
    )
    .expect("shared welcome pointer route should dispatch recovery recent project");
    assert_eq!(
        dispatched.pointer.route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Recover,
        })
    );
    assert_eq!(
        dispatched.event,
        Some(WelcomeHostEvent::RecoverRecentProject {
            path: "E:/Projects/demo-01".to_string(),
        })
    );
}

#[test]
fn shared_welcome_recent_pointer_bridge_dispatches_safe_action() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWelcomeSurfaceTemplateBridge::new().expect("builtin welcome bridge should build");
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(welcome_layout(2));
    let item_index = 1usize;
    let safe = welcome_recent_row_geometry(
        welcome_recent_viewport(UiSize::new(720.0, 620.0)),
        item_index,
        0.0,
    )
    .safe;
    let dispatched = dispatch_shared_welcome_recent_pointer_click(
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(safe.x + safe.width * 0.5, safe.y + safe.height * 0.5),
    )
    .expect("shared welcome pointer route should dispatch safe recent project");
    assert_eq!(
        dispatched.pointer.route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Safe,
        })
    );
    assert_eq!(
        dispatched.event,
        Some(WelcomeHostEvent::SafeRecentProject {
            path: "E:/Projects/demo-01".to_string(),
        })
    );
}

#[test]
fn shared_welcome_recent_pointer_bridge_preserves_state_for_unchanged_layout() {
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    let layout = welcome_layout(8);

    assert!(!pointer_bridge.sync(layout.clone()));
    assert!(!pointer_bridge.sync(layout));
    assert_eq!(pointer_bridge.state(), WelcomeRecentPointerState::default());
}

#[test]
fn welcome_recent_pointer_routes_large_lists_without_materializing_rows() {
    let layout = welcome_layout(10_000);
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(layout.clone());

    let list_point = UiPoint::new(120.0, 190.0);
    let scrolled = pointer_bridge.handle_scroll(list_point, 310_000.0);
    let item_index = scrolled
        .state
        .hovered_item_index
        .expect("post-scroll pointer position should resolve an arithmetic row");
    assert!(item_index > 1_000);
    assert_eq!(scrolled.route, Some(WelcomeRecentPointerRoute::ListSurface));

    let viewport = layout.viewport;
    let geometry = welcome_recent_row_geometry(viewport, item_index, scrolled.state.scroll_offset);
    let remove_point = UiPoint::new(
        geometry.remove.x + geometry.remove.width * 0.5,
        geometry.remove.y + geometry.remove.height * 0.5,
    );
    let moved = pointer_bridge.handle_move(remove_point);
    assert_eq!(
        moved.route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Remove,
        })
    );
    let clicked = pointer_bridge.handle_click(remove_point);
    assert_eq!(clicked.route, moved.route);

    let open_point = UiPoint::new(
        geometry.open.x + geometry.open.width * 0.5,
        geometry.open.y + geometry.open.height * 0.5,
    );
    assert_eq!(
        pointer_bridge.handle_click(open_point).route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Open,
        })
    );

    let recover_point = UiPoint::new(
        geometry.recover.x + geometry.recover.width * 0.5,
        geometry.recover.y + geometry.recover.height * 0.5,
    );
    assert_eq!(
        pointer_bridge.handle_click(recover_point).route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Recover,
        })
    );

    let safe_point = UiPoint::new(
        geometry.safe.x + geometry.safe.width * 0.5,
        geometry.safe.y + geometry.safe.height * 0.5,
    );
    assert_eq!(
        pointer_bridge.handle_click(safe_point).route,
        Some(WelcomeRecentPointerRoute::Action {
            item_index,
            action: WelcomeRecentPointerAction::Safe,
        })
    );

    let next_geometry =
        welcome_recent_row_geometry(viewport, item_index + 1, scrolled.state.scroll_offset);
    let row_gap_point = UiPoint::new(
        geometry.row.x + geometry.row.width * 0.5,
        (geometry.row.bottom() + next_geometry.row.y) * 0.5,
    );
    let gap_move = pointer_bridge.handle_move(row_gap_point);
    assert_eq!(gap_move.route, Some(WelcomeRecentPointerRoute::ListSurface));
    assert_eq!(gap_move.state.hovered_item_index, None);
    assert_eq!(gap_move.state.hovered_action, None);

    pointer_bridge.handle_move(remove_point);
    let gap_scroll = pointer_bridge.handle_scroll(row_gap_point, 0.0);
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
    let scene_entries = harness.runtime.editor_snapshot().scene_entries;
    assert!(
        scene_entries.len() >= 2,
        "default fixture should expose hierarchy rows"
    );

    let mut pointer_bridge = HierarchyPointerBridge::new();
    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 40.0,
            item_count: scene_entries.len(),
        },
        HierarchyPointerState::default(),
    );

    let scrolled = pointer_bridge.handle_scroll(UiPoint::new(120.0, 20.0), 24.0);
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 40.0,
            item_count: scene_entries.len(),
        },
        scrolled.state,
    );
    let dispatched = dispatch_shared_hierarchy_pointer_click(
        &harness.runtime,
        &mut pointer_bridge,
        &scene_entries,
        UiPoint::new(80.0, 28.0),
    )
    .expect("shared hierarchy pointer route should dispatch scene-node selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(HierarchyPointerRoute::Node { item_index: 1 })
    );
    assert_eq!(dispatched.selected_entity, Some(scene_entries[1].entity));
    let effects = dispatched
        .effects
        .expect("hierarchy node click should dispatch into runtime");
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
            world_domain: crate::core::play::WorldDomain::Edit,
            node_id: scene_entries[1].entity,
        })
    );
}

#[test]
fn shared_hierarchy_pointer_bridge_skips_rebuild_for_unchanged_layout_and_state() {
    let mut pointer_bridge = HierarchyPointerBridge::new();
    let layout = HierarchyPointerLayout {
        pane_width: 260.0,
        pane_height: 40.0,
        item_count: 2,
    };
    let state = HierarchyPointerState::default();

    assert!(pointer_bridge.sync(layout.clone(), state.clone()));
    assert!(!pointer_bridge.sync(layout, state));
}

#[test]
fn hierarchy_pointer_routes_large_lists_without_materializing_rows() {
    let mut pointer_bridge = HierarchyPointerBridge::new();
    pointer_bridge.sync(
        HierarchyPointerLayout {
            pane_width: 260.0,
            pane_height: 120.0,
            item_count: 10_000,
        },
        HierarchyPointerState::default(),
    );

    let point = UiPoint::new(80.0, 20.0);
    let scrolled = pointer_bridge.handle_scroll(point, 135_000.0);
    let Some(HierarchyPointerRoute::Node { item_index }) = scrolled.route else {
        panic!("the post-scroll pointer position should resolve an arithmetic row route");
    };
    assert!(item_index > 1_000);

    let clicked = pointer_bridge.handle_click(point);
    assert_eq!(
        clicked.route,
        Some(HierarchyPointerRoute::Node { item_index })
    );
    assert_eq!(
        pointer_bridge.handle_move(UiPoint::new(2.0, point.y)).route,
        Some(HierarchyPointerRoute::ListSurface)
    );
}

fn welcome_layout(count: usize) -> WelcomeRecentPointerLayout {
    let pane_size = UiSize::new(720.0, 620.0);
    WelcomeRecentPointerLayout {
        viewport: welcome_recent_viewport(pane_size),
        recent_project_paths: (0..count)
            .map(|index| format!("E:/Projects/demo-{index:02}"))
            .collect(),
    }
}
