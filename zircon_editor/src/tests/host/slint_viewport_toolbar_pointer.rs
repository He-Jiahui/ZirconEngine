use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_viewport_toolbar_pointer_click, BuiltinViewportToolbarTemplateBridge,
};
use crate::ui::slint_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
    ViewportToolbarPointerRoute,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::{EditorEvent, EditorViewportEvent};
use zircon_scene::DisplayMode;
use zircon_ui::{UiPoint, UiSize};

#[test]
fn shared_viewport_toolbar_pointer_bridge_routes_controls_from_shared_hit_test() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    let route = bridge
        .handle_click(
            "scene.main",
            "tool.scale",
            120.0,
            0.0,
            40.0,
            20.0,
            UiPoint::new(132.0, 10.0),
        )
        .unwrap();
    assert_eq!(
        route.route,
        Some(ViewportToolbarPointerRoute::SetTool {
            surface_key: "scene.main".to_string(),
            tool: "Scale".to_string(),
        })
    );
}

#[test]
fn shared_viewport_toolbar_pointer_click_dispatches_display_cycle_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_viewport_toolbar_pointer_display");
    let template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    let dispatched = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "scene.main",
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
        EditorEvent::Viewport(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::WireOverlay,
        })
    );
}

#[test]
fn shared_viewport_toolbar_pointer_click_falls_back_to_surface_projection_when_control_rect_is_empty(
) {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_viewport_toolbar_projection_fallback");
    let mut template_bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");
    template_bridge
        .recompute_layout(UiSize::new(1280.0, 28.0))
        .expect("viewport toolbar layout should compute");
    let mut pointer_bridge = ViewportToolbarPointerBridge::new();
    pointer_bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    let dispatched = dispatch_shared_viewport_toolbar_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        "scene.main",
        "display.cycle",
        0.0,
        0.0,
        0.0,
        0.0,
        UiPoint::new(300.0, 10.0),
    )
    .expect("shared viewport toolbar route should fall back to projected control frame");

    assert_eq!(
        dispatched.pointer.route,
        Some(ViewportToolbarPointerRoute::CycleDisplayMode {
            surface_key: "scene.main".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("projection-backed click should dispatch into the runtime");
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::WireOverlay,
        })
    );
}

#[test]
fn shared_viewport_toolbar_surface_replaces_legacy_direct_click_routes() {
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
        "callback set_tool(tool: string);",
        "callback set_transform_space(space: string);",
        "callback set_projection_mode(mode: string);",
        "callback align_view(orientation: string);",
        "callback set_display_mode(mode: string);",
        "callback set_grid_mode(mode: string);",
        "callback set_translate_snap(step: float);",
        "callback set_rotate_snap_degrees(step: float);",
        "callback set_scale_snap(step: float);",
        "callback set_preview_lighting(enabled: bool);",
        "callback set_preview_skybox(enabled: bool);",
        "callback set_gizmos_enabled(enabled: bool);",
        "callback frame_selection();",
        "callback viewport_set_tool(",
        "callback viewport_set_transform_space(",
        "callback viewport_set_projection_mode(",
        "callback viewport_align_view(",
        "callback viewport_set_display_mode(",
        "callback viewport_set_grid_mode(",
        "callback viewport_set_translate_snap(",
        "callback viewport_set_rotate_snap_degrees(",
        "callback viewport_set_scale_snap(",
        "callback viewport_set_preview_lighting(",
        "callback viewport_set_preview_skybox(",
        "callback viewport_set_gizmos_enabled(",
        "callback viewport_frame_selection(",
        "viewport_set_tool(tool) =>",
        "viewport_set_transform_space(space) =>",
        "viewport_set_projection_mode(mode) =>",
        "viewport_align_view(orientation) =>",
        "viewport_set_display_mode(mode) =>",
        "viewport_set_grid_mode(mode) =>",
        "viewport_set_translate_snap(step) =>",
        "viewport_set_rotate_snap_degrees(step) =>",
        "viewport_set_scale_snap(step) =>",
        "viewport_set_preview_lighting(enabled) =>",
        "viewport_set_preview_skybox(enabled) =>",
        "viewport_set_gizmos_enabled(enabled) =>",
        "viewport_frame_selection() =>",
        "clicked => { root.set_tool(\"Drag\"); }",
        "clicked => { root.set_transform_space(\"Local\"); }",
        "clicked => { root.frame_selection(); }",
    ] {
        assert!(
            !workbench.contains(needle),
            "viewport toolbar still exposes legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "callback viewport_toolbar_pointer_clicked(",
        "pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle) || chrome.contains(needle),
            "viewport toolbar shared pointer hook `{needle}` is missing"
        );
    }

    for needle in [
        "ui.on_viewport_set_tool(",
        "ui.on_viewport_set_transform_space(",
        "ui.on_viewport_set_projection_mode(",
        "ui.on_viewport_frame_selection(",
    ] {
        assert!(
            !app.contains(needle),
            "slint host app should no longer register direct viewport toolbar callback `{needle}`"
        );
    }

    assert!(
        app.contains("ui.on_viewport_toolbar_pointer_clicked(")
            || wiring.contains("ui.on_viewport_toolbar_pointer_clicked("),
        "slint host app must register shared viewport toolbar callback"
    );
}
