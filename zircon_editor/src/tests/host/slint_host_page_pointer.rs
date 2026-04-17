use crate::host::slint_host::callback_dispatch::{
    dispatch_shared_host_page_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::host::slint_host::host_page_pointer::{
    build_workbench_host_page_pointer_layout, WorkbenchHostPagePointerBridge,
    WorkbenchHostPagePointerItem, WorkbenchHostPagePointerLayout, WorkbenchHostPagePointerRoute,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{EditorEvent, LayoutCommand, MainPageId, WorkbenchChromeMetrics};
use zircon_ui::{UiFrame, UiPoint, UiSize};

#[test]
fn shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test() {
    let mut bridge = WorkbenchHostPagePointerBridge::new();
    bridge.sync(sample_host_page_layout());

    let route = bridge
        .handle_click(1, 80.0, 92.0, UiPoint::new(90.0, 12.0))
        .unwrap();
    assert_eq!(
        route.route,
        Some(WorkbenchHostPagePointerRoute::Tab {
            item_index: 1,
            page_id: "inspector".to_string(),
        })
    );
}

#[test]
fn shared_host_page_pointer_click_dispatches_activate_main_page_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_activate");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = crate::WorkbenchViewModel::build(&chrome);
    let mut pointer_bridge = WorkbenchHostPagePointerBridge::new();
    let root_frames = template_bridge.root_shell_frames();
    pointer_bridge.sync(build_workbench_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
    ));

    let dispatched = dispatch_shared_host_page_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        0,
        8.0,
        92.0,
        UiPoint::new(12.0, 12.0),
    )
    .expect("shared host page route should dispatch activate main page");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchHostPagePointerRoute::Tab {
            item_index: 0,
            page_id: MainPageId::workbench().0,
        })
    );
    let effects = dispatched
        .effects
        .expect("host page click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::workbench(),
        })
    );
}

#[test]
fn shared_host_page_pointer_layout_prefers_shared_shell_width_over_metric_strip_estimate() {
    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_shared_width");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let chrome = harness.runtime.chrome_snapshot();
    let model = crate::WorkbenchViewModel::build(&chrome);
    let root_frames = template_bridge.root_shell_frames();
    let layout = build_workbench_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(&root_frames),
    );

    assert_eq!(
        layout.strip_frame,
        UiFrame::new(0.0, 26.0, 1280.0, 24.0),
        "shared shell projection should own the root host-page strip width"
    );
}

#[test]
fn shared_host_page_pointer_layout_prefers_shared_host_strip_frame_over_shell_metric_estimate() {
    let harness = EventRuntimeHarness::new("zircon_slint_host_page_pointer_shared_strip");
    let chrome = harness.runtime.chrome_snapshot();
    let model = crate::WorkbenchViewModel::build(&chrome);
    let layout = build_workbench_host_page_pointer_layout(
        &model,
        &WorkbenchChromeMetrics::default(),
        Some(
            &crate::host::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames {
                shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
                host_page_strip_frame: Some(UiFrame::new(40.0, 54.0, 1110.0, 28.0)),
                ..Default::default()
            },
        ),
    );

    assert_eq!(
        layout.strip_frame,
        UiFrame::new(40.0, 54.0, 1110.0, 28.0),
        "shared host-page strip projection should outrank the shell-level metric estimate"
    );
}

#[test]
fn shared_host_page_surface_replaces_legacy_direct_click_route() {
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
        !workbench.contains("clicked => { root.activate_host_page(page.id); }"),
        "workbench shell still exposes legacy direct host page callback"
    );
    assert!(
        workbench.contains("callback host_page_pointer_clicked("),
        "workbench shell must expose shared host page pointer callback"
    );
    assert!(
        chrome.contains("callback pointer_pressed("),
        "DockTabButton should expose raw pointer coordinates for shared hit-test"
    );
    assert!(
        !app.contains("ui.on_activate_host_page("),
        "slint host app should no longer register direct host page activation callback"
    );
    assert!(
        app.contains("ui.on_host_page_pointer_clicked(")
            || wiring.contains("ui.on_host_page_pointer_clicked("),
        "slint host app must register shared host page pointer callback"
    );
}

fn sample_host_page_layout() -> WorkbenchHostPagePointerLayout {
    WorkbenchHostPagePointerLayout {
        strip_frame: UiFrame::new(0.0, 26.0, 1280.0, 24.0),
        items: vec![
            WorkbenchHostPagePointerItem {
                page_id: MainPageId::workbench().0,
            },
            WorkbenchHostPagePointerItem {
                page_id: "inspector".to_string(),
            },
        ],
    }
}
