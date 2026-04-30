fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_viewport_surface_uses_unified_rust_pointer_dispatch() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let viewport = source("src/ui/slint_host/app/viewport.rs");

    assert!(globals.contains("on_viewport_pointer_event"));
    assert!(wiring.contains("pane_surface_host.on_viewport_pointer_event("));
    assert!(viewport.contains("dispatch_viewport_pointer_event("));
    for legacy in [
        "on_viewport_pointer_moved",
        "on_viewport_left_pressed",
        "on_viewport_scrolled",
        "InputManager",
    ] {
        assert!(
            !wiring.contains(legacy) && !viewport.contains(legacy),
            "viewport path should not keep legacy callback `{legacy}`"
        );
    }
}
