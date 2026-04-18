use crate::ui::slint_host::callback_dispatch::{
    dispatch_builtin_asset_surface_control, dispatch_shared_document_tab_close_pointer_click,
    dispatch_shared_document_tab_pointer_click, BuiltinAssetSurfaceTemplateBridge,
    BuiltinWorkbenchTemplateBridge,
};
use crate::ui::slint_host::document_tab_pointer::{
    build_workbench_document_tab_pointer_layout, WorkbenchDocumentTabPointerBridge,
    WorkbenchDocumentTabPointerItem, WorkbenchDocumentTabPointerLayout,
    WorkbenchDocumentTabPointerRoute, WorkbenchDocumentTabPointerSurface,
};
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{
    compute_workbench_shell_geometry, default_preview_fixture, DocumentNode, EditorEvent,
    FloatingWindowLayout, LayoutCommand, MainPageId, NativeWindowHostState, ShellFrame,
    ShellSizePx, TabStackLayout, ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId,
    WorkbenchChromeMetrics, WorkbenchViewModel,
};
use zircon_ui::{UiEventKind, UiFrame, UiPoint, UiSize};

#[test]
fn shared_document_tab_pointer_bridge_routes_main_and_floating_tab_targets() {
    let mut bridge = WorkbenchDocumentTabPointerBridge::new();
    bridge.sync(sample_document_tab_layout());

    let main = bridge
        .handle_activate_click("main", 1, 110.0, 120.0, UiPoint::new(132.0, 14.0))
        .unwrap();
    assert_eq!(
        main.route,
        Some(WorkbenchDocumentTabPointerRoute::ActivateTab {
            surface_key: "main".to_string(),
            item_index: 1,
            instance_id: "editor.game#1".to_string(),
        })
    );

    let floating_close = bridge
        .handle_close_click("preview", 0, 8.0, 122.0, UiPoint::new(106.0, 14.0))
        .unwrap();
    assert_eq!(
        floating_close.route,
        Some(WorkbenchDocumentTabPointerRoute::CloseTab {
            surface_key: "preview".to_string(),
            item_index: 0,
            instance_id: "editor.preview#1".to_string(),
        })
    );
}

#[test]
fn shared_document_tab_pointer_click_dispatches_focus_view_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_document_tab_pointer_activate");
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
    let mut pointer_bridge = WorkbenchDocumentTabPointerBridge::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[],
    );
    pointer_bridge.sync(build_workbench_document_tab_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        None,
        &floating_window_projection_bundle,
    ));

    let dispatched = dispatch_shared_document_tab_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "main",
        0,
        8.0,
        114.0,
        UiPoint::new(24.0, 14.0),
    )
    .expect("shared document tab route should dispatch focus view");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchDocumentTabPointerRoute::ActivateTab {
            surface_key: "main".to_string(),
            item_index: 0,
            instance_id: "editor.scene#1".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("document tab click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: crate::ViewInstanceId::new("editor.scene#1"),
        })
    );
}

#[test]
fn shared_document_tab_close_pointer_click_dispatches_close_view_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_document_tab_pointer_close");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let asset_surface_bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface template bridge should build");
    dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &asset_surface_bridge,
        "OpenAssetBrowser",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("asset browser open control should resolve")
    .expect("asset browser should open into the runtime");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let (close_index, close_tab) = model
        .document_tabs
        .iter()
        .enumerate()
        .find(|(_, tab)| tab.closeable)
        .expect("opened asset browser should add a closeable document tab");
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &harness.runtime.current_layout(),
        &harness.runtime.descriptors(),
        ShellSizePx::new(1280.0, 720.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    let mut pointer_bridge = WorkbenchDocumentTabPointerBridge::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[],
    );
    pointer_bridge.sync(build_workbench_document_tab_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        None,
        &floating_window_projection_bundle,
    ));

    let dispatched = dispatch_shared_document_tab_close_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "main",
        close_index,
        8.0 + close_index as f32 * 160.0,
        114.0,
        UiPoint::new(8.0 + close_index as f32 * 160.0 + 96.0, 14.0),
    )
    .expect("shared document tab close route should dispatch close view");

    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchDocumentTabPointerRoute::CloseTab {
            surface_key: "main".to_string(),
            item_index: close_index,
            instance_id: close_tab.instance_id.0.clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("document tab close should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::CloseView {
            instance_id: close_tab.instance_id.clone(),
        })
    );
}

#[test]
fn shared_document_tab_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "clicked => { root.activate_document_tab(tab.id); }",
        "close_clicked => { root.close_tab(tab.id); }",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy document tab callback `{needle}`"
        );
    }

    for needle in [
        "callback document_tab_pointer_clicked(",
        "callback document_tab_close_pointer_clicked(",
        "pointer_clicked(x, y) =>",
        "close_pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle) || chrome.contains(needle),
            "document tab shared pointer hook `{needle}` is missing"
        );
    }

    for needle in ["ui.on_activate_document_tab(", "ui.on_close_tab("] {
        assert!(
            !app.contains(needle),
            "slint host app should no longer register direct document tab callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_document_tab_pointer_clicked(",
        "ui.on_document_tab_close_pointer_clicked(",
    ] {
        assert!(
            app.contains(needle) || wiring.contains(needle),
            "slint host app must register shared document tab callback `{needle}`"
        );
    }
}

#[test]
fn shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1440.0, 900.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [640.0, 320.0, 700.0, 420.0],
        }],
    );
    let layout = build_workbench_document_tab_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        None,
        &floating_window_projection_bundle,
    );
    let floating_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == window_id.0)
        .expect("floating window strip should exist");

    assert_eq!(
        floating_surface.strip_frame,
        UiFrame::new(
            640.0,
            320.0,
            700.0,
            WorkbenchChromeMetrics::default().document_header_height,
        ),
        "floating tab strip should follow native host bounds when they are available"
    );
}

fn sample_document_tab_layout() -> WorkbenchDocumentTabPointerLayout {
    WorkbenchDocumentTabPointerLayout {
        surfaces: vec![
            WorkbenchDocumentTabPointerSurface {
                key: "main".to_string(),
                strip_frame: UiFrame::new(312.0, 51.0, 640.0, 31.0),
                items: vec![
                    WorkbenchDocumentTabPointerItem {
                        instance_id: "editor.scene#1".to_string(),
                        closeable: true,
                    },
                    WorkbenchDocumentTabPointerItem {
                        instance_id: "editor.game#1".to_string(),
                        closeable: true,
                    },
                ],
            },
            WorkbenchDocumentTabPointerSurface {
                key: "preview".to_string(),
                strip_frame: UiFrame::new(100.0, 140.0, 360.0, 31.0),
                items: vec![WorkbenchDocumentTabPointerItem {
                    instance_id: "editor.preview#1".to_string(),
                    closeable: true,
                }],
            },
        ],
    }
}
