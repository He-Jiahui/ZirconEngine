use crate::core::editor_event::{EditorEvent, EditorViewportEvent};
use crate::scene::viewport::DisplayMode;
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_shared_viewport_toolbar_pointer_click, BuiltinViewportToolbarTemplateBridge,
};
use crate::ui::slint_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
    ViewportToolbarPointerRoute,
};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

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
