use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
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
use crate::{EditorEvent, SelectionHostEvent, WelcomeHostEvent};
use zircon_runtime::ui::{layout::UiPoint, layout::UiSize};

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

#[test]
fn shared_list_surfaces_do_not_expose_legacy_direct_callback_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let panes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/panes.slint"
    ));
    let welcome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/welcome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));

    for needle in [
        "callback hierarchy_select(",
        "callback asset_select_folder(",
        "callback asset_select_item(",
        "callback asset_activate_reference(",
        "callback welcome_open_recent_project(",
        "callback welcome_remove_recent_project(",
        "hierarchy_select(node_id) =>",
        "asset_select_folder(folder_id) =>",
        "asset_select_item(asset_uuid) =>",
        "asset_activate_reference(asset_uuid) =>",
        "welcome_open_recent_project(path) =>",
        "welcome_remove_recent_project(path) =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback select(node_id: string);",
        "callback select_folder(folder_id: string);",
        "callback select_asset(asset_uuid: string);",
        "callback activate_reference(asset_uuid: string);",
        "select_folder(folder_id) =>",
        "select_asset(asset_uuid) =>",
        "activate_reference(asset_uuid) =>",
    ] {
        let found = assets.contains(needle) || panes.contains(needle);
        assert!(
            !found,
            "asset or pane leaf surface still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback open_recent_project(path: string);",
        "callback remove_recent_project(path: string);",
    ] {
        assert!(
            !welcome.contains(needle),
            "welcome leaf surface still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_hierarchy_select(",
        "ui.on_welcome_open_recent_project(",
        "ui.on_welcome_remove_recent_project(",
        "ui.on_asset_select_folder(",
        "ui.on_asset_select_item(",
        "ui.on_asset_activate_reference(",
        "fn select_hierarchy_node(",
        "fn select_asset_folder(",
        "fn select_asset_item(",
        "fn activate_asset_reference(",
    ] {
        assert!(
            !app.contains(needle),
            "slint host app still registers legacy direct callback `{needle}`"
        );
    }
}

#[test]
fn welcome_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let welcome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/welcome.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback welcome_project_name_edited(",
        "callback welcome_location_edited(",
        "callback welcome_create_project(",
        "callback welcome_open_existing_project(",
        "welcome_project_name_edited(value) =>",
        "welcome_location_edited(value) =>",
        "welcome_create_project() =>",
        "welcome_open_existing_project() =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy welcome business callback `{needle}`"
        );
    }

    for needle in [
        "callback project_name_edited(value: string);",
        "callback location_edited(value: string);",
        "callback create_project();",
        "callback open_existing_project();",
        "edited(value) => { root.project_name_edited(value); }",
        "edited(value) => { root.location_edited(value); }",
        "clicked => { root.create_project(); }",
        "clicked => { root.open_existing_project(); }",
    ] {
        assert!(
            !welcome.contains(needle),
            "welcome leaf surface still exposes legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_welcome_project_name_edited(",
        "ui.on_welcome_location_edited(",
        "ui.on_welcome_create_project(",
        "ui.on_welcome_open_existing_project(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy welcome control callback `{needle}`"
        );
    }

    for needle in [
        "callback welcome_control_changed(control_id: string, value: string);",
        "callback welcome_control_clicked(control_id: string);",
        "control_changed(control_id, value) => { root.welcome_control_changed(control_id, value); }",
        "control_clicked(control_id) => { root.welcome_control_clicked(control_id); }",
    ] {
        assert!(
            pane_surface.contains(needle),
            "pane surface catalog is missing generic welcome control route `{needle}`"
        );
    }

    for needle in [
        "callback control_changed(control_id: string, value: string);",
        "callback control_clicked(control_id: string);",
        "edited(value) => { root.control_changed(\"ProjectNameEdited\", value); }",
        "edited(value) => { root.control_changed(\"LocationEdited\", value); }",
        "clicked => { root.control_clicked(\"CreateProject\"); }",
        "clicked => { root.control_clicked(\"OpenExistingProject\"); }",
    ] {
        assert!(
            welcome.contains(needle),
            "welcome leaf surface is missing generic control route `{needle}`"
        );
    }

    for needle in [
        "pane_surface_host.on_welcome_control_changed(",
        "pane_surface_host.on_welcome_control_clicked(",
    ] {
        assert!(
            wiring.contains(needle),
            "slint host wiring is missing generic welcome control callback `{needle}`"
        );
    }
}

#[test]
fn pane_surface_actions_use_generic_template_callbacks_instead_of_legacy_menu_action_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let lifecycle = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/host_lifecycle.rs"
    ));

    for needle in [
        "callback menu_action(action_id: string);",
        "callback empty_action(action_id: string);",
        "trigger_action(action_id) => { root.empty_action(action_id); }",
        "open_assets() => { root.empty_action(\"OpenView.editor.assets\"); }",
    ] {
        let found = workbench.contains(needle) || assets.contains(needle);
        assert!(
            !found,
            "pane surfaces still expose legacy menu-action callback `{needle}`"
        );
    }

    for needle in ["ui.on_menu_action(", "fn handle_menu_action("] {
        let found = wiring.contains(needle) || lifecycle.contains(needle);
        assert!(
            !found,
            "slint host still wires legacy menu-action callback `{needle}`"
        );
    }

    for needle in [
        "export { PaneSurfaceHostContext } from \"workbench/pane_surface.slint\";",
        "PaneSurfaceHostContext.surface_control_clicked(\"TriggerAction\", action_id);",
        "PaneSurfaceHostContext.surface_control_clicked(control_id, action_id);",
    ] {
        assert!(
            workbench.contains(needle) || pane_surface.contains(needle),
            "workbench shell is missing generic pane-surface control route `{needle}`"
        );
    }

    for needle in [
        "trigger_action(action_id) => { PaneSurfaceHostContext.surface_control_clicked(\"TriggerAction\", action_id); }",
        "clicked => { root.surface_control_clicked(\"TriggerAction\", \"OpenView.editor.assets\"); }",
    ] {
        let found =
            workbench.contains(needle) || pane_surface.contains(needle) || assets.contains(needle);
        assert!(
            found,
            "pane leaf surfaces are missing generic control route `{needle}`"
        );
    }

    assert!(
        wiring.contains("let pane_surface_host = ui.global::<PaneSurfaceHostContext>();"),
        "slint host wiring must access the exported PaneSurfaceHostContext global"
    );
    assert!(
        wiring.contains("pane_surface_host.on_surface_control_clicked("),
        "slint host wiring is missing generic pane-surface control global callback"
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
