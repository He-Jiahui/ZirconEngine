fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn scene_and_game_viewport_surfaces_keep_distinct_pointer_ownership() {
    let globals = source("src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs");
    let scene_viewport = source("src/ui/retained_host/app/viewport/pointer_event.rs");
    let game_viewport = source("src/ui/retained_host/app/viewport/game_input.rs");

    assert!(globals.contains("on_scene_viewport_pointer_event"));
    assert!(globals.contains("on_game_viewport_pointer_event"));
    assert!(wiring.contains("pane_surface_host.on_scene_viewport_pointer_event("));
    assert!(wiring.contains("pane_surface_host.on_game_viewport_pointer_event("));
    assert!(scene_viewport.contains("dispatch_viewport_pointer_event("));
    assert!(game_viewport.contains("route_play_preview_input(runtime_event)"));
    for legacy in [
        "on_viewport_pointer_event",
        "on_viewport_pointer_moved",
        "on_viewport_left_pressed",
        "on_viewport_scrolled",
        "InputManager",
    ] {
        assert!(
            !wiring.contains(legacy)
                && !scene_viewport.contains(legacy)
                && !game_viewport.contains(legacy),
            "viewport path should not keep legacy callback `{legacy}`"
        );
    }
}

#[test]
fn shared_viewport_surface_reuses_one_pointer_dispatcher() {
    let bridge = source("src/ui/retained_host/callback_dispatch/viewport/bridge.rs");
    let dispatch = source("src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs");

    assert!(bridge.contains("dispatcher: UiPointerDispatcher"));
    assert!(bridge.contains("dispatcher: viewport_pointer_dispatcher()"));
    assert!(!dispatch.contains("let dispatcher = viewport_pointer_dispatcher();"));
}
