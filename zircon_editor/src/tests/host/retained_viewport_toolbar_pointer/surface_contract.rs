fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_viewport_toolbar_surface_uses_toml_controls_and_rust_callbacks() {
    let globals = source("src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs");
    let viewport = source("src/ui/retained_host/app/viewport/toolbar_pointer/click.rs");
    let toolbar = source("assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml");

    assert!(globals.contains("on_viewport_toolbar_pointer_clicked"));
    assert!(wiring.contains("pane_surface_host.on_viewport_toolbar_pointer_clicked("));
    assert!(viewport.contains("viewport_toolbar_pointer_clicked"));
    for required in [
        "SetTool",
        "SetTransformSpace",
        "SetDisplayMode",
        "SetGridMode",
        "FrameSelection",
    ] {
        assert!(
            toolbar.contains(required),
            "viewport toolbar asset missing `{required}`"
        );
    }
}

#[test]
fn viewport_toolbar_pointer_bridge_uses_route_intent_only() {
    let bridge =
        source("src/ui/retained_host/viewport_toolbar_pointer/viewport_toolbar_pointer_bridge.rs");
    let rebuild = source("src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs");
    let dispatch = source("src/ui/retained_host/viewport_toolbar_pointer/dispatch_event.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::ViewportToolbar"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("viewport_toolbar_route_for_pointer_dispatch"));
    for forbidden in [
        "targets:",
        "ViewportToolbarPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "viewport toolbar pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}
