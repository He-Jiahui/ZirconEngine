fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_viewport_toolbar_surface_uses_toml_controls_and_rust_callbacks() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let viewport = source("src/ui/slint_host/app/viewport.rs");
    let toolbar = source("assets/ui/editor/host/scene_viewport_toolbar.ui.toml");

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
        assert!(toolbar.contains(required), "viewport toolbar asset missing `{required}`");
    }
}
