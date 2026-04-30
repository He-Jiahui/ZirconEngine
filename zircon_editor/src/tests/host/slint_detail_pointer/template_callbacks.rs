fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn inspector_surface_controls_use_pane_surface_host_callbacks() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let inspector = source("src/ui/slint_host/app/inspector.rs");

    for required in [
        "on_inspector_control_changed",
        "on_inspector_control_clicked",
        "invoke_inspector_control_changed",
        "invoke_inspector_control_clicked",
    ] {
        assert!(globals.contains(required), "host globals missing `{required}`");
    }
    assert!(wiring.contains("pane_surface_host.on_inspector_control_changed("));
    assert!(wiring.contains("pane_surface_host.on_inspector_control_clicked("));
    assert!(inspector.contains("dispatch_inspector_control_changed"));
    assert!(inspector.contains("dispatch_inspector_control_clicked"));
}
