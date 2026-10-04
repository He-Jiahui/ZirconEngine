// 核对共享视口工具栏指针点击进入运行时分发器并切换显示模式。
use std::collections::BTreeSet;

use crate::core::editor_event::{
    EditorEvent, EditorEventSource, EditorViewportEvent, MenuAction, ViewInstanceId,
};
use crate::scene::viewport::DisplayMode;
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_shared_viewport_toolbar_pointer_click, BuiltinViewportToolbarTemplateBridge,
};
use crate::ui::retained_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
    ViewportToolbarPointerRoute,
};
use crate::ui::workbench::layout::{
    LayoutCommand, MainPageId, SplitAxis, SplitPlacement, WorkspaceTarget,
};
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime_interface::ui::layout::UiPoint;

#[test]
fn shared_viewport_toolbar_pointer_click_dispatches_display_cycle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_viewport_toolbar_pointer_display");
    let template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));
    let view_id = ViewInstanceId::new("editor.scene#1");

    let dispatched = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "scene.main",
        &view_id,
        "display.cycle",
        244.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(252.0, 10.0),
    )
    .expect("shared viewport toolbar route should dispatch display mode change");

    assert_eq!(
        dispatched.pointer.route,
        Some(ViewportToolbarPointerRoute::CycleDisplayMode {
            surface_key: "scene.main".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("viewport toolbar click should dispatch into the runtime");
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::ForView {
            view_id,
            event: Box::new(EditorViewportEvent::SetDisplayMode {
                mode: DisplayMode::WireOverlay,
            }),
        })
    );
}

#[test]
fn real_workbench_scene_split_routes_each_toolbar_to_its_leaf_and_rejects_retired_target() {
    let _guard = env_lock().lock().unwrap();

    let mut harness = EventRuntimeHarness::new("zircon_retained_viewport_toolbar_scene_split");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("the event fixture should expose the editor manager");
    let first = manager
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.scene"))
        .map(|instance| instance.instance_id)
        .expect("the real workbench should start with one Scene leaf");
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(
                crate::core::editor_event::ViewDescriptorId::new("editor.scene"),
            )),
        )
        .expect("opening Scene through the owner event path should succeed");
    let second = manager
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.scene")
                && instance.instance_id != first
        })
        .map(|instance| instance.instance_id)
        .expect("the owner event path should allocate a second real Scene instance");
    assert_ne!(
        first, second,
        "opening Scene should allocate a new leaf identity"
    );
    let first_core = ViewInstanceId::new(first.0.clone());
    let second_core = ViewInstanceId::new(second.0.clone());

    manager
        .apply_layout_command(LayoutCommand::CreateSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: Vec::new(),
            axis: SplitAxis::Horizontal,
            placement: SplitPlacement::After,
            new_instance: second.clone(),
        })
        .expect("the second Scene should be inserted into a real workbench split");
    manager
        .apply_layout_command(LayoutCommand::ResizeSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: Vec::new(),
            ratio: 0.37,
        })
        .expect("the split ratio should be persisted on the workbench layout");

    // These keys are the committed presentation keys produced by
    // scene_projection::document_leaves, and the active tabs prove which leaf
    // instance each surface resolves to.
    let layout = manager.current_layout();
    let workspace = layout
        .content_workspace_for_page(&MainPageId::workbench())
        .expect("the workbench document workspace should exist");
    let (left_surface_key, right_surface_key) = match workspace {
        crate::ui::workbench::layout::DocumentNode::SplitNode {
            first: left,
            second: right,
            ratio,
            ..
        } => {
            assert_eq!(*ratio, 0.37);
            let crate::ui::workbench::layout::DocumentNode::Tabs(left_tabs) = left.as_ref() else {
                panic!("the first Scene surface must be a document leaf");
            };
            let crate::ui::workbench::layout::DocumentNode::Tabs(right_tabs) = right.as_ref()
            else {
                panic!("the second Scene surface must be a document leaf");
            };
            assert_eq!(left_tabs.active_tab.as_ref(), Some(&first));
            assert_eq!(right_tabs.active_tab.as_ref(), Some(&second));
            (
                format!("document:{}", left.node_id()),
                format!("document:{}", right.node_id()),
            )
        }
        _ => panic!("the real Scene leaves must be represented by a split node"),
    };

    // The Workbench owner restores both committed leaves before any retained
    // render pass. The executor below only consumes these retained sessions and
    // cannot fork a missing/retired target.
    manager
        .focus_view(&first)
        .expect("the workbench should retain the first Scene leaf as focused");
    let saved_sessions = harness.runtime.scene_viewport_workspace_sessions();
    harness
        .runtime
        .restore_scene_viewport_workspace_sessions(&saved_sessions, Some(&second_core));
    let restored_sessions = harness.runtime.scene_viewport_workspace_sessions();
    assert_eq!(restored_sessions.len(), 2);
    assert!(restored_sessions.contains_key(&first_core));
    assert!(restored_sessions.contains_key(&second_core));

    let bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout([
        left_surface_key.as_str(),
        right_surface_key.as_str(),
    ]));

    dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        &left_surface_key,
        &first_core,
        "align.pos_x",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    )
    .expect("left toolbar should target the first committed Scene leaf");
    dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        &right_surface_key,
        &second_core,
        "align.neg_z",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    )
    .expect("right toolbar should target the second committed Scene leaf");

    let after_routes = harness.runtime.scene_viewport_workspace_sessions();
    assert_eq!(
        manager.current_focused_view(),
        Some(first.clone()),
        "targeted toolbar routing must not move workbench focus"
    );
    let left_camera = after_routes
        .get(&first_core)
        .and_then(|session| session.camera.as_ref())
        .expect("left Scene leaf should retain its camera");
    let right_camera = after_routes
        .get(&second_core)
        .and_then(|session| session.camera.as_ref())
        .expect("right Scene leaf should retain its camera");
    assert_ne!(
        left_camera.transform, right_camera.transform,
        "independent toolbar alignment must preserve distinct leaf cameras"
    );

    // Retiring one retained session must reject its old route without changing
    // the other leaf, focus, or the session map.
    harness
        .runtime
        .retain_scene_viewports(&BTreeSet::from([first_core.clone()]));
    let before_stale = harness.runtime.scene_viewport_workspace_sessions();
    let before_stale_journal = harness.runtime.journal().records().len();
    let stale = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        &right_surface_key,
        &second_core,
        "display.cycle",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    );
    assert!(stale
        .expect_err("a retired Scene leaf must reject toolbar dispatch")
        .contains("stale"));
    assert_eq!(
        harness.runtime.scene_viewport_workspace_sessions(),
        before_stale,
        "a stale toolbar event must leave the retained Scene state unchanged"
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        before_stale_journal,
        "a stale toolbar event must be rejected before journal mutation"
    );
    assert_eq!(
        manager.current_focused_view(),
        Some(first.clone()),
        "stale routing must not move workbench focus"
    );

    // A missing identity is rejected before journaling/execution and cannot
    // fall back to the active Scene session.
    let missing = ViewInstanceId::new("editor.scene#missing");
    let before_missing = harness.runtime.scene_viewport_workspace_sessions();
    let before_missing_journal = harness.runtime.journal().records().len();
    let missing_result = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        &left_surface_key,
        &missing,
        "align.pos_y",
        0.0,
        0.0,
        48.0,
        20.0,
        UiPoint::new(4.0, 4.0),
    );
    assert!(missing_result
        .expect_err("a missing Scene leaf must reject toolbar dispatch")
        .contains("stale"));
    assert_eq!(
        harness.runtime.scene_viewport_workspace_sessions(),
        before_missing,
        "a missing toolbar target must leave all live Scene state unchanged"
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        before_missing_journal,
        "a missing toolbar target must be rejected before journal mutation"
    );
    assert_eq!(
        manager.current_focused_view(),
        Some(first),
        "missing routing must not move workbench focus"
    );
}
