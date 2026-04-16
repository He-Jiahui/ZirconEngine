use crate::host::slint_host::callback_dispatch::{
    dispatch_shared_drawer_header_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::host::slint_host::drawer_header_pointer::{
    build_workbench_drawer_header_pointer_layout, WorkbenchDrawerHeaderPointerBridge,
    WorkbenchDrawerHeaderPointerItem, WorkbenchDrawerHeaderPointerLayout,
    WorkbenchDrawerHeaderPointerRoute, WorkbenchDrawerHeaderPointerSurface,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{
    compute_workbench_shell_geometry, ActivityDrawerMode, ActivityDrawerSlot, EditorEvent,
    LayoutCommand, ShellSizePx, WorkbenchChromeMetrics, WorkbenchViewModel,
};
use zircon_ui::{UiFrame, UiPoint, UiSize};

#[test]
fn shared_drawer_header_pointer_bridge_routes_group_tabs_from_shared_hit_test() {
    let mut bridge = WorkbenchDrawerHeaderPointerBridge::new();
    bridge.sync(sample_drawer_header_layout());

    let route = bridge
        .handle_click("left", 1, 112.0, 96.0, UiPoint::new(120.0, 12.0))
        .unwrap();
    assert_eq!(
        route.route,
        Some(WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key: "left".to_string(),
            item_index: 1,
            slot: "left_bottom".to_string(),
            instance_id: "editor.hierarchy#1".to_string(),
        })
    );
}

#[test]
fn shared_drawer_header_pointer_click_dispatches_drawer_toggle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_drawer_header_pointer_toggle");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &harness.runtime.current_layout(),
        &harness.runtime.descriptors(),
        ShellSizePx::new(1280.0, 720.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    let mut pointer_bridge = WorkbenchDrawerHeaderPointerBridge::new();
    pointer_bridge.sync(build_workbench_drawer_header_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
    ));

    let dispatched = dispatch_shared_drawer_header_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "left",
        0,
        6.0,
        96.0,
        UiPoint::new(24.0, 12.0),
    )
    .expect("shared drawer header route should dispatch drawer toggle");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key: "left".to_string(),
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("drawer header click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::Collapsed,
        })
    );
}

#[test]
fn shared_drawer_header_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
    ));

    assert!(
        !workbench.contains("clicked => { root.toggle_drawer_tab(tab.slot, tab.id); }"),
        "workbench shell still exposes legacy direct drawer header callback"
    );
    for needle in [
        "callback drawer_header_pointer_clicked(",
        "pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle) || chrome.contains(needle),
            "drawer header shared pointer hook `{needle}` is missing"
        );
    }
    assert!(
        !app.contains("ui.on_toggle_drawer_tab("),
        "slint host app should no longer register direct drawer toggle callback"
    );
    assert!(
        app.contains("ui.on_drawer_header_pointer_clicked(")
            || wiring.contains("ui.on_drawer_header_pointer_clicked("),
        "slint host app must register shared drawer header callback"
    );
}

fn sample_drawer_header_layout() -> WorkbenchDrawerHeaderPointerLayout {
    WorkbenchDrawerHeaderPointerLayout {
        surfaces: vec![
            WorkbenchDrawerHeaderPointerSurface {
                key: "left".to_string(),
                strip_frame: UiFrame::new(35.0, 53.0, 240.0, 25.0),
                items: vec![
                    WorkbenchDrawerHeaderPointerItem {
                        slot: "left_top".to_string(),
                        instance_id: "editor.project#1".to_string(),
                    },
                    WorkbenchDrawerHeaderPointerItem {
                        slot: "left_bottom".to_string(),
                        instance_id: "editor.hierarchy#1".to_string(),
                    },
                ],
            },
            WorkbenchDrawerHeaderPointerSurface {
                key: "right".to_string(),
                strip_frame: UiFrame::new(1002.0, 53.0, 240.0, 25.0),
                items: vec![WorkbenchDrawerHeaderPointerItem {
                    slot: "right_top".to_string(),
                    instance_id: "editor.inspector#1".to_string(),
                }],
            },
        ],
    }
}
