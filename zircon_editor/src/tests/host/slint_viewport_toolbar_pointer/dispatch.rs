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
use zircon_runtime::ui::layout::UiPoint;

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
