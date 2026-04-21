use crate::core::editor_event::EditorEvent;
use crate::core::editor_event::SelectionHostEvent;
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::binding_dispatch::WelcomeHostEvent;
use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_hierarchy_pointer_click, dispatch_shared_welcome_recent_pointer_click,
    BuiltinWelcomeSurfaceTemplateBridge,
};
use crate::ui::slint_host::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerRoute, HierarchyPointerState,
};
use crate::ui::slint_host::welcome_recent_pointer::{
    WelcomeRecentPointerAction, WelcomeRecentPointerBridge, WelcomeRecentPointerLayout,
    WelcomeRecentPointerRoute, WelcomeRecentPointerState,
};
use zircon_runtime::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_welcome_recent_pointer_bridge_scrolls_and_dispatches_remove_action() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWelcomeSurfaceTemplateBridge::new().expect("builtin welcome bridge should build");
    let mut pointer_bridge = WelcomeRecentPointerBridge::new();
    pointer_bridge.sync(welcome_layout(8), WelcomeRecentPointerState::default());

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(120.0, 190.0), 140.0)
        .expect("welcome recent list should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(welcome_layout(8), scrolled.state.clone());
    let item_index = 3usize;
    let click_y = 112.0 + item_index as f32 * 122.0 - scrolled.state.scroll_offset + 92.0;
    let dispatched = dispatch_shared_welcome_recent_pointer_click(
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(168.0, click_y),
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
fn shared_hierarchy_pointer_bridge_scrolls_and_dispatches_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_hierarchy_pointer");
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

fn welcome_layout(count: usize) -> WelcomeRecentPointerLayout {
    WelcomeRecentPointerLayout {
        pane_size: UiSize::new(720.0, 620.0),
        recent_project_paths: (0..count)
            .map(|index| format!("E:/Projects/demo-{index:02}"))
            .collect(),
    }
}
