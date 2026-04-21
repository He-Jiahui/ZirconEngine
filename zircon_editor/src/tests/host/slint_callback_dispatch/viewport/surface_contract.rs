#[test]
fn shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_surface_host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let viewport = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/viewport.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback viewport_pointer_moved(",
        "callback viewport_left_pressed(",
        "callback viewport_left_released(",
        "callback viewport_right_pressed(",
        "callback viewport_right_released(",
        "callback viewport_middle_pressed(",
        "callback viewport_middle_released(",
        "callback viewport_scrolled(",
        "viewport_pointer_moved(x, y) =>",
        "viewport_left_pressed(x, y) =>",
        "viewport_left_released() =>",
        "viewport_right_pressed(x, y) =>",
        "viewport_right_released() =>",
        "viewport_middle_pressed(x, y) =>",
        "viewport_middle_released() =>",
        "viewport_scrolled(delta) =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy direct viewport callback `{needle}`"
        );
    }

    assert!(
        !workbench.contains(
            "export { PaneSurfaceHostContext } from \"workbench/pane_surface_host_context.slint\";"
        ),
        "workbench shell should stop re-exporting the shared PaneSurfaceHostContext global"
    );
    assert!(
        pane_surface_host_context.contains("export global PaneSurfaceHostContext {"),
        "pane surface host context owner must keep the shared PaneSurfaceHostContext global"
    );

    for needle in [
        "ui.on_viewport_pointer_moved(",
        "ui.on_viewport_left_pressed(",
        "ui.on_viewport_left_released(",
        "ui.on_viewport_right_pressed(",
        "ui.on_viewport_right_released(",
        "ui.on_viewport_middle_pressed(",
        "ui.on_viewport_middle_released(",
        "ui.on_viewport_scrolled(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy direct viewport callback `{needle}`"
        );
    }

    assert!(
        wiring.contains("pane_surface_host.on_viewport_pointer_event("),
        "slint host wiring must register unified shared viewport callback on the exported global"
    );

    for needle in [
        "InputManager",
        "InputButton",
        "InputEvent",
        "submit_event(InputEvent::CursorMoved",
        "submit_event(InputEvent::ButtonPressed",
        "submit_event(InputEvent::ButtonReleased",
        "submit_event(InputEvent::WheelScrolled",
    ] {
        assert!(
            !app.contains(needle) && !viewport.contains(needle),
            "slint viewport host still depends on legacy raw input manager path `{needle}`"
        );
    }

    assert!(
        viewport.contains("dispatch_viewport_pointer_event("),
        "slint viewport host must dispatch through shared viewport pointer bridge"
    );
}
