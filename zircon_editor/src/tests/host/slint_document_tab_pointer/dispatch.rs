use crate::core::editor_event::{
    EditorEvent, LayoutCommand as EventLayoutCommand, ViewInstanceId as EventViewInstanceId,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_builtin_asset_surface_control, dispatch_shared_document_tab_close_pointer_click,
    dispatch_shared_document_tab_pointer_click, BuiltinAssetSurfaceTemplateBridge,
    BuiltinHostWindowTemplateBridge,
};
use crate::ui::slint_host::document_tab_pointer::{
    build_host_document_tab_pointer_layout, HostDocumentTabPointerBridge,
    HostDocumentTabPointerItem, HostDocumentTabPointerLayout, HostDocumentTabPointerRoute,
    HostDocumentTabPointerSurface,
};
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime::ui::{
    binding::UiEventKind,
    layout::{UiFrame, UiPoint, UiSize},
};

#[test]
fn shared_document_tab_pointer_bridge_routes_main_and_floating_tab_targets() {
    let mut bridge = HostDocumentTabPointerBridge::new();
    bridge.sync(sample_document_tab_layout());

    let main = bridge
        .handle_activate_click("main", 1, 110.0, 120.0, UiPoint::new(132.0, 14.0))
        .unwrap();
    assert_eq!(
        main.route,
        Some(HostDocumentTabPointerRoute::ActivateTab {
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
        Some(HostDocumentTabPointerRoute::CloseTab {
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
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
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
    let mut pointer_bridge = HostDocumentTabPointerBridge::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[],
    );
    pointer_bridge.sync(build_host_document_tab_pointer_layout(
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
        Some(HostDocumentTabPointerRoute::ActivateTab {
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
        EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.scene#1"),
        })
    );
}

#[test]
fn shared_document_tab_close_pointer_click_dispatches_close_view_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_document_tab_pointer_close");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
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
    let mut pointer_bridge = HostDocumentTabPointerBridge::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[],
    );
    pointer_bridge.sync(build_host_document_tab_pointer_layout(
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
        Some(HostDocumentTabPointerRoute::CloseTab {
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
        EditorEvent::Layout(EventLayoutCommand::CloseView {
            instance_id: EventViewInstanceId::new(close_tab.instance_id.0.clone()),
        })
    );
}

fn sample_document_tab_layout() -> HostDocumentTabPointerLayout {
    HostDocumentTabPointerLayout {
        surfaces: vec![
            HostDocumentTabPointerSurface {
                key: "main".to_string(),
                strip_frame: UiFrame::new(312.0, 51.0, 640.0, 31.0),
                items: vec![
                    HostDocumentTabPointerItem {
                        instance_id: "editor.scene#1".to_string(),
                        closeable: true,
                    },
                    HostDocumentTabPointerItem {
                        instance_id: "editor.game#1".to_string(),
                        closeable: true,
                    },
                ],
            },
            HostDocumentTabPointerSurface {
                key: "preview".to_string(),
                strip_frame: UiFrame::new(100.0, 140.0, 360.0, 31.0),
                items: vec![HostDocumentTabPointerItem {
                    instance_id: "editor.preview#1".to_string(),
                    closeable: true,
                }],
            },
        ],
    }
}
